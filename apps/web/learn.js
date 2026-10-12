import { BOOK, prayers, wudu, topics, words } from './guide-data.js';
import { quran } from './quran-data.js';

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
  return link(`Guide · printed p. ${pages}`, `${BOOK}#page=${Number.isFinite(first) ? first + 3 : 4}`);
}
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
  card.append(el('h3', word.title));
  if (word.verses) {
    for (const verse of word.verses) {
      const row = el('div', undefined, 'verse');
      const arabic = el('p', verse.arabic, 'arabic'); arabic.lang = 'ar'; arabic.dir = 'rtl';
      row.append(el('span', verse.ayah, 'verse-number'), arabic, el('p', verse.translit, 'translit'), el('p', verse.meaning, 'meaning'));
      card.append(row);
    }
    card.append(el('p', 'English: Muhammad A. S. Abdel Haleem. Arabic and translation verified through quran.ai, 11 October 2026.', 'source-caption'));
    card.append(link('Read the Qur’an source', word.url));
  } else {
    const arabic = el('p', word.arabic, 'arabic'); arabic.lang = 'ar'; arabic.dir = 'rtl';
    card.append(arabic, el('p', word.translit, 'translit'), el('p', word.meaning, 'meaning'));
    if (word.note) card.append(el('p', word.note, 'recitation-note'));
    card.append(citation(word.pages));
    if (word.link) card.append(el('span', ' · '), link(...word.link));
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
  $('#prayer-kicker').textContent = `${prayer.time} / ${prayer.units} obligatory units`;
  $('#prayer-name').textContent = `${prayer.name}, with understanding.`;
  $('#prayer-description').textContent = prayer.voice;
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
  add(0,'Before the opening takbīr','Confirm purification, covering, prayer time, direction and space. Know which prayer you intend in your heart. This walkthrough describes a complete individual prayer, following the guide’s sequence.', [], '6–8, 17–23','conditions');
  for (let unit = 1; unit <= selectedPrayer.units; unit++) {
    if (unit === 1) add(unit,'Begin with the takbīr','Stand when able. Raise your hands to shoulder or ear level as you say the opening takbīr. Place the right hand over the left at the chest and look toward your place of prostration.', ['takbir'],'23–25','standing');
    add(unit,unit === 1 ? 'The opening & recitation' : 'Stand and recite',unit === 1 ? 'Read an opening supplication, then seek refuge with Allah. Recite Al-Fātiḥah, with the basmalah, and then another portion of Qur’an you know. Al-Ikhlāṣ below is an example, not a required choice.' : unit === 2 ? 'Stand after the brief rest. Recite Al-Fātiḥah and another portion of Qur’an. Do not repeat the opening supplication. The guide describes this unit as shorter than the first.' : 'Rise for this unit and recite Al-Fātiḥah. The guide describes the later units with Al-Fātiḥah alone, more briefly than the first two.', [...(unit === 1 ? ['opening','refuge'] : []),'fatihah',...(unit <= 2 ? ['ikhlas'] : [])], '26–30, 38, 43','fatihah');
    add(unit,'Bow · rukūʿ','Raise your hands with the takbīr, then bow. Place your palms on your knees with the back and head level as able. Settle in the posture and glorify Allah. Do not recite Qur’an here.',['takbir','ruku'],'30–32','bowing');
    add(unit,'Rise & settle','Rise fully upright, raising your hands. Say the first phrase while rising, then praise Allah once upright. Remain still before moving on. The guide leaves the hands at the sides here; Ibn Bāz differs.',['rise','praise'],'32–33','bowing');
    add(unit,'The first prostration','Say the takbīr and lower into sujūd. Forehead and nose, hands, knees and toes touch the ground; keep the forearms lifted. The guide places hands before knees; this is a documented point of difference. Settle, glorify Allah and make supplication.',['takbir','sujud'],'33–36','prostration');
    add(unit,'Sit between the prostrations','Rise with the takbīr and sit calmly. The guide places the left foot beneath you and keeps the right foot upright with toes toward the Qiblah, as able. Ask Allah for forgiveness.',['takbir','forgive'],'36–37','prostration');
    add(unit,'The second prostration','Say the takbīr and make a second prostration like the first, with the same remembrance. Give it time; there is no timed countdown to follow.',['takbir','sujud'],'37–38','prostration');
    if (unit === 2 || unit === selectedPrayer.units) {
      const final = unit === selectedPrayer.units;
      add(unit,final ? 'The final sitting' : 'The first tashahhud',final ? `Rise with the takbīr and sit for tashahhud and blessings upon the Prophet. ${selectedPrayer.units > 2 ? 'The guide describes tawarruk in this final sitting when able, without crowding others.' : 'Keep the earlier sitting posture in this two-unit prayer.'} Point the right index finger; the guide describes moving it and looking at it.` : 'Rise with the takbīr and sit for tashahhud. Point the right index finger; the guide describes moving it and looking at it. It includes blessings upon the Prophet in this first sitting too. You will then rise with the takbīr for the third unit.', ['takbir','tashahhud','salawat'],'39–46','sittings');
    } else if (unit < selectedPrayer.units) {
      add(unit,'A brief rest, then rise','After the second prostration, sit briefly until settled, then rise with the takbīr for the next unit. The guide describes supporting the rise on the hands in closed fists. Adapt to your ability.',['takbir'],'38, 42–43','sittings');
    }
  }
  add(selectedPrayer.units,'Seek protection & supplicate','After the tashahhud and blessings, seek protection from the four trials. You may then make further supplication, using authentic supplications you know.',['protection'],'46–47','finish');
  add(selectedPrayer.units,'Close with peace','Turn the head to the right and say the salutation, then turn to the left and say it again. This completes the prayer sequence.',['salam'],'47–49','finish');
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
    article.append(el('p', 'A LITTLE MORE FAMILIAR', 'eyebrow'), el('h2','Carry what you learned.'), el('p',`You have explored the ${selectedPrayer.units}-unit ${selectedPrayer.name} prayer. Revisit any words you want to practise and seek a teacher’s help with pronunciation. This completion records no worship.`),link('Revisit the words','#words'),el('p','May Allah accept your prayer and make learning easy.','closing-line'));
  } else {
    const step = steps[stepIndex];
    article.append(el('p', `${selectedPrayer.name.toUpperCase()} / ${step.unit ? `UNIT ${step.unit} OF ${selectedPrayer.units}` : 'PREPARATION'}`, 'eyebrow'),el('h2',step.title),el('p',step.body,'step-instruction'));
    const source = el('p',undefined,'step-source');source.append(citation(step.pages),el('span',' · '),link('Context & differences',`#library/${step.topic}`));article.append(source);
    for (const id of step.recite) {
      const detail = el('details', undefined, 'step-recitation');
      // Short phrases stay open; longer recitations remain one tap away.
      detail.open = !['fatihah','ikhlas','opening','refuge','tashahhud','salawat'].includes(id);
      detail.append(el('summary',byId(id).title),wordCard(byId(id),true));article.append(detail);
    }
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
  const target=$('#wudu-content');target.replaceChildren(el('span',String(wuduIndex+1).padStart(2,'0'),'wudu-number'),el('h3',title),el('p',body),citation('8–11'));
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
    const refs=el('div',undefined,'topic-references');refs.append(citation(topic.pages));for(const reference of topic.links||[])refs.append(link(...reference));
    refs.append(link('Link to this topic',`#library/${topic.id}`));body.append(refs);detail.append(body);$('#topic-list').append(detail);
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
  if(name==='library'&&topicId) {
    $('#topic-search').value='';category=topicId==='differences'?'Differences':'All';renderTopics(topicId);
    const target=document.getElementById(`topic-${topicId}`);
    if(target){target.scrollIntoView({block:'start'});target.querySelector('summary').focus({preventScroll:true});return;}
  }
  if(!initial){$('#main-content').focus({preventScroll:true});window.scrollTo({top:0,behavior:'instant'});}
}
window.addEventListener('hashchange',()=>route());route(true);
$('.skip-link').addEventListener('click',event=>{event.preventDefault();$('#main-content').focus();});
