import { BOOK, prayers, wudu, topics, words } from './guide-data.js';
import { quran, evidenceVerses } from './quran-data.js';

const $ = selector => document.querySelector(selector);
const el = (tag, text, cls) => {
  const n = document.createElement(tag);
  if (text !== undefined) n.textContent = text;
  if (cls) n.className = cls;
  return n;
};
const link = (label, href) => { const n = el('a', label); n.href = href; return n; };
function citation(pages) {
  const first = Number.parseInt(pages, 10);
  return link(`Prayer guide · pp. ${pages}`, `${BOOK}#page=${Number.isFinite(first) ? first + 3 : 4}`);
}
function verseCard(reference, excerpt = false) {
  const verse = evidenceVerses.find(v => v.ayah === reference);
  if (!verse) return el('div');
  const card = el('figure', undefined, 'scripture-card');
  card.append(el('p', `FROM THE QUR’AN / ${verse.name.toUpperCase()} ${verse.ayah}`, 'eyebrow'));
  const arabic = el('p', verse.text, 'arabic'); arabic.lang = 'ar'; arabic.dir = 'rtl';
  card.append(arabic, el('blockquote', excerpt && verse.excerpt ? verse.excerpt : verse.meaning, 'scripture-translation'));
  const caption = el('figcaption');
  caption.append(link(`Qur’an · ${verse.name} ${verse.ayah}`, verse.href), el('span', `Translation of the meaning${excerpt && verse.excerpt ? ' (excerpt)' : ''} · ${verse.translator}`));
  card.append(caption); return card;
}
function sourceDisclosure(pages, references = []) {
  const detail = el('details', undefined, 'evidence-disclosure');
  detail.append(el('summary', 'References & further reading'));
  const list = el('div', undefined, 'topic-references');
  for (const reference of references) list.append(link(...reference));
  if (pages) list.append(citation(pages));
  detail.append(list); return detail;
}
$('#home-verse').append(verseCard('13:28', true));
// All data is bundled and inserted as text, never as HTML or executable content.
const normalized = text => text.normalize('NFKD').replace(/[\u0300-\u036f\u064b-\u065f\u0670]/g, '').replace(/[ʿʾ‘’']/g, '').toLowerCase();
const alias = {wudhu:'wudu',wuzu:'wudu',ablution:'wudu',fatiha:'fatihah',sajdah:'prostration',sujud:'prostration',ruku:'bowing',namaz:'prayer',forgiveness:'forgive'};
function searchText(value) { return normalized(value).split(/\s+/).filter(Boolean).map(s => alias[s] || s); }
function matches(value, haystack) { const text = normalized(haystack); return searchText(value).every(term => text.includes(term)); }
let selectedPrayer = prayers[0];
let stepIndex = 0;
let steps = [];
let category = 'All';
let timesLoading;
function ensureTimes() {
  // Learning remains usable without downloading or starting the calculation engine.
  timesLoading ??= Promise.all([import('./app.js'), import('./offline.js')]).catch(() => {
    $('#engine-state').textContent = 'The calculation files could not load. Reload the complete app to retry. The learning sections remain available.';
  });
}

function wordCard(word, compact = false) {
  const card = el('article', undefined, compact ? 'recitation compact' : 'recitation');
  if (!compact) card.id = `word-${word.id}`;
  card.append(el('h3', word.title));
  if (word.verses) {
    for (const verse of word.verses) {
      const row = el('div', undefined, 'verse');
      const arabic = el('p', verse.arabic, 'arabic'); arabic.lang = 'ar'; arabic.dir = 'rtl';
      row.append(el('span', verse.ayah, 'verse-number'), arabic, el('p', verse.translit, 'translit'), el('p', verse.meaning, 'meaning'));
      card.append(row);
    }
    card.append(el('p', 'Translation of the meaning · Muhammad A. S. Abdel Haleem', 'source-caption'));
    card.append(link('Read the Qur’an source', word.url));
  } else {
    const arabic = el('p', word.arabic, 'arabic'); arabic.lang = 'ar'; arabic.dir = 'rtl';
    card.append(arabic, el('p', word.translit, 'translit'), el('p', word.meaning, 'meaning'));
    if (word.note) card.append(el('p', word.note, 'recitation-note'));
    card.append(sourceDisclosure(word.pages, word.link ? [word.link] : []));
  }
  return card;
}
const recitations = [
  ...words.slice(0, 3),
  { id:'fatihah',title:'Al-Fātiḥah · the Opening',verses:quran.filter(v => v.ayah.startsWith('1:')),url:'https://quran.com/1?translations=85'},
  { id:'ikhlas',title:'Al-Ikhlāṣ · an example short sūrah',verses:quran.filter(v => v.ayah.startsWith('112:')),url:'https://quran.com/112?translations=85'},
  ...words.slice(3),
];
const byId = id => recitations.find(w => w.id === id);

function selectPrayer(prayer) {
  selectedPrayer = prayer;
  for (const button of $('#prayer-picker').children) button.setAttribute('aria-pressed', String(button.dataset.prayer === prayer.id));
  $('#prayer-kicker').textContent = `${prayer.time} / ${prayer.units} rakʿahs`;
  $('#prayer-name').textContent = `${prayer.name}, step by step.`;
  $('#prayer-description').textContent = prayer.voice;
  $('#prayer-reading').textContent = prayer.reading;
}
for (const prayer of prayers) {
  const button = el('button'); button.type = 'button'; button.dataset.prayer = prayer.id;
  const ar = el('span', prayer.arabic, 'prayer-ar'); ar.lang = 'ar'; ar.dir = 'rtl';
  const counts = el('span', `${prayer.units} units`, 'unit-label');
  counts.append(el('span', ` · ${prayer.time}`, 'prayer-time-label'));
  button.append(ar, el('strong', prayer.name), counts);
  button.addEventListener('click', () => selectPrayer(prayer));
  $('#prayer-picker').append(button);
}
selectPrayer(selectedPrayer);

function buildSteps() {
  const list = [];
  const add = (unit, title, body, recite, pages, topic) => list.push({ unit, title, body, recite, pages, topic });
  add(0,'Before the opening takbīr','Preparation includes purification, suitable covering, prayer time, the Qiblah and a clean space. The worshipper knows which prayer is being offered and holds the intention sincerely for Allah.', [], '6–8, 17–23','conditions');
  for (let unit = 1; unit <= selectedPrayer.units; unit++) {
    if (unit === 1) add(unit,'Begin with the takbīr','The worshipper stands as able, raises the hands to shoulder or ear level and says Allāhu akbar. The right hand is placed over the left at the chest, and the gaze is directed toward the place of prostration.', ['takbir'],'23–25','standing');
    add(unit,unit === 1 ? 'The opening & recitation' : 'Stand and recite',unit === 1 ? 'An opening supplication is followed by seeking refuge with Allah. Al-Fātiḥah is recited, beginning with the basmalah, then another known passage of Qur’an. Al-Ikhlāṣ below is one example, rather than a required choice.' : unit === 2 ? 'After the brief sitting, the worshipper stands and recites Al-Fātiḥah, then another known passage of Qur’an. The opening supplication is not repeated.' : 'Al-Fātiḥah is recited in this rakʿah. The recitation is given time and attention, without hurried movement.', [...(unit === 1 ? ['opening','refuge'] : []),'fatihah',...(unit <= 2 ? ['ikhlas'] : [])], '26–30, 38, 43','fatihah');
    add(unit,'Bow · rukūʿ','The hands are raised with the takbīr, and the worshipper bows with palms on the knees. The back and head remain level as able. Allah is glorified while the posture settles; Qur’an is not recited in bowing.',['takbir','ruku'],'30–32','bowing');
    add(unit,'Rise & settle','The worshipper rises fully upright, raising the hands. The words of rising are said during the movement, then Allah is praised once upright. The standing posture settles before the next movement.',['rise','praise'],'32–33','bowing');
    add(unit,'The first prostration','With the takbīr, the worshipper lowers into sujūd. The forehead and nose, hands, knees and toes meet the ground as able, with the forearms lifted. Allah is glorified, and supplication may be made.',['takbir','sujud'],'33–36','prostration');
    add(unit,'Sit between the prostrations','The worshipper rises with the takbīr and sits calmly, with the left foot beneath and the right foot upright as able. Forgiveness is sought with the words Rabbighfir lī.',['takbir','forgive'],'36–37','prostration');
    add(unit,'The second prostration','A second prostration follows with the takbīr. Allah is glorified again, with time to settle in the posture. The two prostrations are part of each rakʿah.',['takbir','sujud'],'37–38','prostration');
    if (unit === 2 || unit === selectedPrayer.units) {
      const final = unit === selectedPrayer.units;
      add(unit,final ? 'The final sitting' : 'The first tashahhud',final ? 'The worshipper rises with the takbīr and sits for the tashahhud and blessings upon the Prophet ﷺ. The posture is adapted to actual ability, with consideration for those nearby.' : 'The worshipper rises with the takbīr and sits for the tashahhud and blessings upon the Prophet ﷺ. The prayer then continues into the third rakʿah.', ['takbir','tashahhud','salawat'],'39–46','sittings');
    } else if (unit < selectedPrayer.units) {
      add(unit,'A brief rest, then rise','A brief sitting follows the second prostration. Once settled, the worshipper rises with the takbīr for the next rakʿah, using support as needed.',['takbir'],'38, 42–43','sittings');
    }
  }
  add(selectedPrayer.units,'Seek protection & supplicate','In the final sitting, Allah’s protection is sought from the four trials named in the reported supplication. Further authentic supplications may follow, with humility and hope.',['protection'],'46–47','finish');
  add(selectedPrayer.units,'Close with peace','The head turns to the right with the closing salutation, then to the left with the same words. This completes the prayer. Remembrance continues afterward.',['salam'],'47–49','finish');
  return list;
}
function prepareWalk() {
  steps = buildSteps(); stepIndex = 0;
  $('#step-jump').replaceChildren(); $('#step-list').replaceChildren();
  steps.forEach((step, index) => {
    const label = `${index + 1}. ${step.unit ? `Unit ${step.unit} · ` : ''}${step.title}`;
    const option = el('option', label); option.value = index; $('#step-jump').append(option);
    const li = el('li'); const button = el('button', label); button.type = 'button';
    button.addEventListener('click', () => { stepIndex = index; renderStep(true); }); li.append(button); $('#step-list').append(li);
  });
  renderStep();
}
function renderStep(focus = false) {
  const article = $('#step-content'); article.replaceChildren();
  const complete = stepIndex === steps.length;
  $('#step-next').textContent = complete ? 'Return to learning' : stepIndex === steps.length - 1 ? 'Finish walkthrough' : 'Next step';
  $('#step-back').disabled = stepIndex === 0;
  $('#step-progress').textContent = complete ? 'Walkthrough complete' : `${stepIndex + 1} of ${steps.length}`;
  $('#step-jump').value = String(Math.min(stepIndex, steps.length - 1));
  [...$('#step-list').children].forEach((li,i) => li.firstChild.setAttribute('aria-current', String(i === stepIndex ? 'step' : 'false')));
  if (complete) {
    article.append(el('p', 'A LITTLE MORE FAMILIAR', 'eyebrow'), el('h2','A little more familiar.'), el('p',`The ${selectedPrayer.units}-unit ${selectedPrayer.name} sequence has been explored. Revisiting the words and learning with a patient teacher can help turn a new sequence into familiar worship.`),link('Remembrance after prayer','#library/after'),link('Return to the words','#words'),el('p','May Allah accept our prayers and make learning easy.','closing-line'));
  } else {
    const step = steps[stepIndex];
    article.append(el('p', `${selectedPrayer.name.toUpperCase()} / ${step.unit ? `UNIT ${step.unit} OF ${selectedPrayer.units}` : 'PREPARATION'}`, 'eyebrow'),el('h2',step.title),el('p',step.body,'step-instruction'));
    const context = topics.find(t => t.id === step.topic);
    article.append(link('Understand this movement', `#library/${step.topic}`));
    for (const id of step.recite) {
      const detail = el('details', undefined, 'step-recitation');
      // Short phrases stay open; longer recitations remain one tap away.
      detail.open = !['fatihah','ikhlas','opening','refuge','tashahhud','salawat'].includes(id);
      detail.append(el('summary',byId(id).title),wordCard(byId(id),true));article.append(detail);
    }
    article.append(sourceDisclosure(step.pages, context?.links || []));
  }
  if (focus) { article.focus({preventScroll:true}); article.scrollIntoView({block:'start',behavior:'instant'}); }
}
$('#start-guide').addEventListener('click',()=>{prepareWalk();location.hash='walk/'+selectedPrayer.id;});
$('#step-back').addEventListener('click',()=>{stepIndex=Math.max(0,stepIndex-1);renderStep(true);});
$('#step-next').addEventListener('click',()=>{if(stepIndex===steps.length){location.hash='learn';return;}stepIndex++;renderStep(true);});
$('#step-jump').addEventListener('change',event=>{stepIndex=Number(event.target.value);renderStep(true);});
$('#seated').addEventListener('change',event=>{$('#ability-note').hidden=!event.target.checked;});

let wuduIndex = 0;
function renderWudu(focus=false) {
  const [title,body] = wudu[wuduIndex];
  const target=$('#wudu-content');target.replaceChildren(el('span',String(wuduIndex+1).padStart(2,'0'),'wudu-number'),el('h3',title),el('p',body),sourceDisclosure('8–11', [['Sahih Muslim 235a', 'https://sunnah.com/muslim:235a']]));
  const controls=el('div',undefined,'wudu-controls');
  for(const [label,direction] of [['Previous',-1],['Next step',1]]) {
    const button=el('button',label);button.type='button';button.disabled=direction===-1 ? wuduIndex===0 : wuduIndex===wudu.length-1;
    button.addEventListener('click',()=>{wuduIndex+=direction;renderWudu(true);});controls.append(button);
  }target.append(controls);
  [...$('#wudu-list').children].forEach((b,i)=>b.setAttribute('aria-pressed',String(i===wuduIndex)));
  if(focus)target.focus({preventScroll:true});
}
wudu.forEach((step,index)=>{const button=el('button',`${String(index+1).padStart(2,'0')} / ${step[2]}`);button.type='button';button.addEventListener('click',()=>{wuduIndex=index;renderWudu(true);});$('#wudu-list').append(button);});
renderWudu();

function renderWords() {
  const found=recitations.filter(w=>matches($('#word-search').value,JSON.stringify(w)));
  $('#word-list').replaceChildren(...found.map(w=>wordCard(w)));
  $('#word-count').textContent=found.length ? `${found.length} recitations · read at your own pace` : 'No recitation found. Try Fatiha, bowing, forgiveness or tashahhud.';
}
$('#word-search').addEventListener('input',renderWords);
for(const [id,cls] of [['show-translit','hide-translit'],['show-meaning','hide-meaning']]) $('#'+id).addEventListener('change',event=>$('#word-list').classList.toggle(cls,!event.target.checked));
renderWords();

const categories=['All',...new Set(topics.map(t=>t.category))];
for(const name of categories) {
  const button=el('button',name);button.type='button';button.addEventListener('click',()=>{category=name;renderTopics();});$('#topic-filters').append(button);
}
function renderTopics(openId) {
  const found=topics.filter(t=>(category==='All'||t.category===category)&&matches($('#topic-search').value,JSON.stringify(t)));
  $('#topic-list').replaceChildren();
  for(const topic of found) {
    const detail=el('details',undefined,'topic');detail.id=`topic-${topic.id}`;detail.open=topic.id===openId;
    const summary=el('summary');summary.append(el('span',topic.category,'topic-category'),el('strong',topic.title),el('span','+','topic-plus'));detail.append(summary);
    const body=el('div',undefined,'topic-body');for(const paragraph of topic.body)body.append(el('p',paragraph));
    for (const reference of topic.verses || []) body.append(verseCard(reference));
    if (topic.related?.length) { const actions=el('nav',undefined,'related-reading');actions.setAttribute('aria-label','Continue learning');for(const action of topic.related)actions.append(link(...action));body.append(actions); }
    body.append(sourceDisclosure(topic.pages,topic.links || []));detail.append(body);$('#topic-list').append(detail);
  }
  $('#topic-count').textContent=found.length ? `${found.length} topics · select a question to read` : 'No matching topic. Try another word or choose All.';
  for(const button of $('#topic-filters').children)button.setAttribute('aria-pressed',String(button.textContent===category));
}
$('#topic-search').addEventListener('input',()=>renderTopics());renderTopics();

function route(initial=false) {
  const [raw,topicId] = location.hash.slice(1).split('/');
  const name=['learn','walk','prepare','words','library','sources','times'].includes(raw)?raw:'learn';
  if(name==='times')ensureTimes();
  if(name==='walk') { const prayer=prayers.find(p=>p.id===topicId); if(prayer&&prayer.id!==selectedPrayer.id){selectPrayer(prayer);prepareWalk();} else if(!steps.length)prepareWalk(); }
  document.querySelectorAll('[data-view]').forEach(section=>section.hidden=section.dataset.view!==name);
  document.querySelectorAll('.app-nav a').forEach(a=>{const active=a.hash===`#${name==='walk'?'learn':name}`;if(active)a.setAttribute('aria-current','page');else a.removeAttribute('aria-current');});
  document.title=`Salah — ${{learn:'Learn with care',walk:`Learn ${selectedPrayer.name}`,prepare:'Prepare for prayer',words:'The words',library:'Your questions',sources:'Sources & approach',times:'Prayer times'}[name]}`;
  if(name==='words'&&topicId) {
    $('#word-search').value='';renderWords();
    const target=document.getElementById(`word-${topicId}`);
    if(target){target.tabIndex=-1;target.scrollIntoView({block:'start'});target.focus({preventScroll:true});return;}
  }
  if(name==='library'&&topicId) {
    $('#topic-search').value='';category='All';const selectedTopic=topicId==='differences'?'importance':topicId;renderTopics(selectedTopic);
    const target=document.getElementById(`topic-${selectedTopic}`);
    if(target){target.scrollIntoView({block:'start'});target.querySelector('summary').focus({preventScroll:true});return;}
  }
  if(!initial){$('#main-content').focus({preventScroll:true});window.scrollTo({top:0,behavior:'instant'});}
}
window.addEventListener('hashchange',()=>route());route(true);
document.querySelectorAll('.reading-shortcuts a, .library-starts a').forEach(anchor => anchor.addEventListener('click', event => {
  if (anchor.hash === location.hash) { event.preventDefault(); route(); }
}));
$('.skip-link').addEventListener('click',event=>{event.preventDefault();$('#main-content').focus();});
