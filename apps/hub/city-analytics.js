// Anonymous aggregate city counts. No cookies, personal app data, URL or coordinates are sent.
(() => {
  const hosts = ['return-istighfar.pages.dev', 'sirat-apps.pages.dev', 'salah-prayer.pages.dev'];
  if (!hosts.includes(location.hostname) || !navigator.onLine || navigator.globalPrivacyControl === true || navigator.doNotTrack === '1') return;
  fetch('https://sirat-insights.shoaibtest2.workers.dev/api/collect-city', {method:'POST', credentials:'omit', keepalive:true}).catch(() => {});
})();
