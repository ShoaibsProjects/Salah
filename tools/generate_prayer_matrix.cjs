// Optional audit tool. Run with TZ=UTC and locally obtained, pinned source files.
// Usage: node generate_prayer_matrix.cjs /path/to/PrayTimes.js /path/to/adhan
const fs = require('node:fs');
const vm = require('node:vm');
const crypto = require('node:crypto');
const path = require('node:path');

const [prayFile, adhanRoot] = process.argv.slice(2);
if (!prayFile || !adhanRoot || process.env.TZ !== 'UTC') {
  throw new Error('Supply PrayTimes.js and adhan package root, and set TZ=UTC');
}
function sha(file) {
  return crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');
}
if (sha(prayFile) !== 'f0e8442410446b1bf2db613bdebd3aa3aef5cdf5483068fbd9212d5acaa180fd') {
  throw new Error('PrayTimes source checksum does not match the audited file');
}
if (sha(path.join(adhanRoot, 'package.json')) !== 'cde0971d595046294f0090cc12ee19566bda96083a5acf6071e4b45ff2496de6') {
  throw new Error('Adhan package checksum does not match the audited package');
}
const cjsDir = path.join(adhanRoot, 'lib/cjs');
const cjsHash = crypto.createHash('sha256');
for (const filename of fs.readdirSync(cjsDir).filter(x => x.endsWith('.js')).sort()) {
  cjsHash.update(filename + '\n');
  cjsHash.update(fs.readFileSync(path.join(cjsDir, filename)));
}
if (cjsHash.digest('hex') !== 'b8825c79bec06c175956761425fc9c3d9326a95b9b341298d6c15fdccef1e6b7') {
  throw new Error('Adhan executable source checksum does not match the audited files');
}
const adhan = require(adhanRoot);
const SolarTime = require(path.join(adhanRoot, 'lib/cjs/SolarTime.js')).default;
if (require(path.join(adhanRoot, 'package.json')).version !== '4.4.6') {
  throw new Error('Unexpected Adhan version');
}
const context = {};
vm.createContext(context);
vm.runInContext(fs.readFileSync(prayFile, 'utf8'), context);

// The explicit offset selects the same local solar cycle as the Rust input.
const cases = [
  ['minneapolis-autumn','ordinary','2026-09-27',44.9778,-93.2650,-300,0,0],
  ['makkah-equinox','ordinary','2026-03-20',21.4225,39.8262,180,0,0],
  ['cape-town-solstice','ordinary','2026-12-21',-33.9249,18.4241,120,0,0],
  ['london-winter','high-lat-existing','2026-12-21',51.5072,-0.1276,0,0,0],
  ['london-summer','missing-twilight','2026-06-21',51.5072,-0.1276,60,0,0],
  ['tromso-winter','polar-night','2026-12-21',69.6492,18.9553,60,0,0],
  // PrayTimes Float returns 29-43 local hours here; subtract one 24-hour cycle.
  // Adhan's UTC-date-based API needs the preceding date for this local cycle.
  ['kiritimati-date-line','date-line','2026-09-27',1.8721,-157.4278,840,-1,-1],
];
const profiles = [
  ['research-15',15,15,'ISNA'],
  ['mwl-angles-18-17',18,17,'MWL'],
];
const keys = ['fajr','dhuhr','asr','maghrib','isha'];
const header = ['case_id','regime','date','latitude','longitude','offset_minutes','profile','fajr_angle','isha_angle','asr','adhan_fajr_kind','adhan_isha_kind'];
for (const key of keys) header.push('praytimes_'+key+'_utc','adhan_'+key+'_utc');
header.push('praytimes_cycle_shift_days','adhan_input_utc_date');
console.log(header.join('\t'));

function iso(date) {
  return date instanceof Date && Number.isFinite(date.getTime()) ? date.toISOString() : 'no_event';
}
for (const [id,regime,dateText,latitude,longitude,offsetMinutes,praytimesCycleShiftDays,adhanDateShiftDays] of cases) {
  const [year,month,day] = dateText.split('-').map(Number);
  const date = new Date(Date.UTC(year, month-1, day));
  const anchor = date.getTime();
  const adhanDate = new Date(anchor + adhanDateShiftDays * 86_400_000);
  for (const [profile,fajrAngle,ishaAngle,method] of profiles) {
    for (const asr of ['Standard','Hanafi']) {
      const pray = new context.PrayTimes(method);
      pray.adjust({fajr:fajrAngle,isha:ishaAngle,dhuhr:'0 min',maghrib:'0 min',asr,highLats:'None'});
      const p = pray.getTimes([year,month,day],[latitude,longitude,0],offsetMinutes/60,0,'Float');
      const params = new adhan.CalculationParameters('Other',fajrAngle,ishaAngle,0,0);
      params.rounding = adhan.Rounding.None;
      params.madhab = asr === 'Hanafi' ? adhan.Madhab.Hanafi : adhan.Madhab.Shafi;
      const coords = new adhan.Coordinates(latitude,longitude);
      const a = new adhan.PrayerTimes(coords,adhanDate,params);
      const solar = new SolarTime(adhanDate,coords);
      const fajrKind = Number.isFinite(solar.hourAngle(-fajrAngle,false)) ? 'angle' : 'fallback';
      const ishaKind = Number.isFinite(solar.hourAngle(-ishaAngle,true)) ? 'angle' : 'fallback';
      const row = [id,regime,dateText,latitude,longitude,offsetMinutes,profile,fajrAngle,ishaAngle,asr,fajrKind,ishaKind];
      for (const key of keys) {
        const localHours = p[key];
        const utc = Number.isFinite(localHours) ? new Date(anchor+(localHours+24*praytimesCycleShiftDays-offsetMinutes/60)*3600000) : null;
        row.push(iso(utc),iso(a[key]));
      }
      row.push(praytimesCycleShiftDays,adhanDate.toISOString().slice(0,10));
      console.log(row.join('\t'));
    }
  }
}
