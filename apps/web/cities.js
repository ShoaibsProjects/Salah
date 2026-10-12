// Reuse the native app's attributed directory. Queries never leave the browser.
const EXPECTED_HASH = '085e06b69c8b0e435177680c05d29bdfbc0cba9722b0a2d3f678478426940878';
const EXPECTED_BYTES = 6806396;
const normalize = text => text.normalize('NFKD').replace(/[\u0300-\u036f]/g, '').toLowerCase();
export function setupCitySearch(onSelect) {
  const input = document.querySelector('#city-search');
  const status = document.querySelector('#city-status');
  const results = document.querySelector('#city-results');
  let directory;
  let pending;
  let revision = 0;
  let timer;
  function applyCity(city) {
    ++revision; input.value = `${city.name}, ${city.country}`; results.replaceChildren();
    status.textContent = `Using ${city.name}, ${city.country}.`;
    onSelect(city);
  }
  async function load() {
    if (directory) return directory;
    if (!pending) pending = (async () => {
      const response = await fetch('./cities-v1.json', { credentials: 'omit' });
      if (!response.ok || response.redirected) throw new Error('The offline city directory could not be loaded. Use coordinates or device location.');
      const bytes = await response.arrayBuffer();
      if (bytes.byteLength !== EXPECTED_BYTES) throw new Error('City directory size check failed. Reload the complete app package.');
      const hash = Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256', bytes)), x => x.toString(16).padStart(2, '0')).join('');
      if (hash !== EXPECTED_HASH) throw new Error('City directory integrity check failed. Use manual coordinates.');
      const data = JSON.parse(new TextDecoder().decode(bytes));
      if (data.schema !== 'salah-offline-cities-v1' || data.cities.length !== 34152) throw new Error('Unsupported city directory.');
      directory = data.cities.map(row => ({ id:row[0], name:row[1], country:data.countries[row[3]] || row[3], countryCode:row[3], latitude:row[4], longitude:row[5], text:normalize([row[1],row[2],data.countries[row[3]],row[3],...row[7]].join(' ')), key:normalize(row[1]) }));
      return directory;
    })().catch(error => { pending = undefined; throw error; });
    return pending;
  }
  input.addEventListener('input', () => {
    const version = ++revision;
    clearTimeout(timer); results.replaceChildren();
    const query = normalize(input.value.trim());
    if (query.length < 2) { status.textContent = 'Type at least two letters. Larger towns and cities are included; smaller places may need coordinates.'; return; }
    status.textContent = 'Searching the local city directory…';
    timer = setTimeout(async () => {
      try {
        const data = await load();
        if (version !== revision) return;
        const terms = query.split(/\s+/);
        const matches = data.filter(city => terms.every(term => city.text.includes(term))).sort((a,b) => Number(b.key === query)-Number(a.key === query) || Number(b.key.startsWith(query))-Number(a.key.startsWith(query))).slice(0,8);
        status.textContent = matches.length ? 'Choose your city from the list.' : 'No city found. Try another spelling.';
        for (const city of matches) {
          const button = document.createElement('button'); button.type = 'button';
          button.textContent = `${city.name}, ${city.country}`;
          button.addEventListener('click', () => applyCity(city));
          results.append(button);
        }
      } catch (error) { if (version === revision) status.textContent = error.message; }
    },180);
  });
  input.disabled = false;
  return {
    async choose(name, country) {
      try {
        const data = await load();
        const matches = data.filter(city => city.key === normalize(name) && city.country === country);
        if (matches.length === 1) applyCity(matches[0]);
        else {
          input.value = name;
          input.dispatchEvent(new Event('input', { bubbles: true }));
        }
      } catch (error) { status.textContent = error.message; }
    },
  };
}
