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
  return link(`Read source · p. ${pages}`, `${BOOK}#page=${Number.isFinite(first) ? first + 3 : 4}`);
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
  $('#prayer-kicker').textContent = `${prayer.time} / ${prayer.units} rakʿahs`;
  $('#prayer-name').textContent = `${prayer.name}, at my own pace.`;
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
  add(0,'Before the opening takbīr','I take a breath and prepare as best I can: purification, covering, prayer time, direction and a clean space. I know in my heart which prayer I am offering for Allah. I do not need to feel perfect before I begin.', [], '6–8, 17–23','conditions');
  for (let unit = 1; unit <= selectedPrayer.units; unit++) {
    if (unit === 1) add(unit,'Begin with the takbīr','I stand as I am able. I raise my hands and say Allāhu akbar. I place my right hand over my left and bring my attention to the prayer. I am turning to Allah, just as I am.', ['takbir'],'23–25','standing');
    add(unit,unit === 1 ? 'The opening & recitation' : 'Stand and recite',unit === 1 ? 'I begin with an opening supplication and seek refuge with Allah. I recite Al-Fātiḥah, then another passage of Qur’an I know. Al-Ikhlāṣ below is one example. I take the words at a steady pace.' : unit === 2 ? 'I stand after the brief rest and recite Al-Fātiḥah, followed by another passage of Qur’an I know. I let the familiar words bring my attention back.' : 'I rise for this unit and recite Al-Fātiḥah. I take a little more time if I need it; learning can happen one prayer at a time.', [...(unit === 1 ? ['opening','refuge'] : []),'fatihah',...(unit <= 2 ? ['ikhlas'] : [])], '26–30, 38, 43','fatihah');
    add(unit,'Bow · rukūʿ','I say Allāhu akbar and bow, placing my hands on my knees as I am able. I settle and glorify Allah. I leave Qur’an recitation for standing, and let this bow remind me of the One who is greater than all.',['takbir','ruku'],'30–32','bowing');
    add(unit,'Rise & settle','I rise fully upright and praise Allah. I pause until I am settled before moving on. Even this small pause can bring me back to the prayer.', ['rise','praise'],'32–33','bowing');
    add(unit,'The first prostration','I say the takbīr and lower into sujūd. I place my forehead and nose, hands, knees and toes on the ground as I am able, keeping my forearms lifted. I settle, glorify Allah and make supplication.', ['takbir','sujud'],'33–36','prostration');
    add(unit,'Sit between the prostrations','I rise with the takbīr and sit calmly, adapting the posture to my ability. I ask Allah to forgive me. I let the words be simple and sincere.', ['takbir','forgive'],'36–37','prostration');
    add(unit,'The second prostration','I say the takbīr and make a second prostration. I glorify Allah and can make supplication. I give myself time; there is no countdown to follow.', ['takbir','sujud'],'37–38','prostration');
    if (unit === 2 || unit === selectedPrayer.units) {
      const final = unit === selectedPrayer.units;
      add(unit,final ? 'The final sitting' : 'The first tashahhud',final ? 'I rise with the takbīr and sit for tashahhud and blessings upon the Prophet ﷺ. I ask Allah for protection and make the supplication I have learned. I sit as I am able, with care for those around me.' : 'I rise with the takbīr and sit for tashahhud and blessings upon the Prophet ﷺ. Then I rise for the third unit, if my prayer continues.', ['takbir','tashahhud','salawat'],'39–46','sittings');
    } else if (unit < selectedPrayer.units) {
      add(unit,'A brief rest, then rise','After the second prostration, I sit briefly until settled, then rise with the takbīr for the next unit. I use the support my body needs and move without rushing.', ['takbir'],'38, 42–43','sittings');
    }
  }
  add(selectedPrayer.units,'Seek protection & supplicate','In my final sitting, I ask Allah to protect me from the trials named in this supplication. I can add another supplication I know, speaking to Allah with humility and hope.', ['protection'],'46–47','finish');
  add(selectedPrayer.units,'Close with peace','I turn my head to the right and offer the salutation, then turn to the left and say it again. I close my prayer with peace, and carry its remembrance with me.', ['salam'],'47–49','finish');
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
    article.append(el('p', 'A LITTLE MORE FAMILIAR', 'eyebrow'), el('h2','Carry what I have learned.'), el('p',`I have explored the ${selectedPrayer.units}-unit ${selectedPrayer.name} prayer. I can revisit any words I want to practise and ask a teacher to help me with pronunciation. Learning here is only preparation; my worship belongs to Allah.`),link('Return to the words','#words'),el('p','May Allah accept my prayer and make learning easy.','closing-line'));
  } else {
    const step = steps[stepIndex];
    article.append(el('p', `${selectedPrayer.name.toUpperCase()} / ${step.unit ? `UNIT ${step.unit} OF ${selectedPrayer.units}` : 'PREPARATION'}`, 'eyebrow'),el('h2',step.title),el('p',step.body,'step-instruction'));
    const source = el('p',undefined,'step-source');source.append(citation(step.pages),el('span',' · '),link('Learn more',`#library/${step.topic}`));article.append(source);
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
    $('#topic-search').value='';category='All';const selectedTopic=topicId==='differences'?'importance':topicId;renderTopics(selectedTopic);
    const target=document.getElementById(`topic-${selectedTopic}`);
    if(target){target.scrollIntoView({block:'start'});target.querySelector('summary').focus({preventScroll:true});return;}
  }
  if(!initial){$('#main-content').focus({preventScroll:true});window.scrollTo({top:0,behavior:'instant'});}
}
window.addEventListener('hashchange',()=>route());route(true);
$('.skip-link').addEventListener('click',event=>{event.preventDefault();$('#main-content').focus();});
