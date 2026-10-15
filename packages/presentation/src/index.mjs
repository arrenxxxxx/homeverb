import enUS from '../locales/en-US.json' with { type: 'json' };
import zhCN from '../locales/zh-CN.json' with { type: 'json' };

export const supportedLocales = Object.freeze(['en-US', 'zh-CN']);
const catalogs = { 'en-US': enUS, 'zh-CN': zhCN };

export function normalizeLocale(locale) {
  const value = String(locale ?? '').replaceAll('_', '-').toLowerCase();
  return value === 'zh-cn' || value === 'zhcn' ? 'zh-CN' : 'en-US';
}

export function createTranslator(requestedLocale = 'en-US') {
  const locale = normalizeLocale(requestedLocale);
  function t(key, parameters = {}) {
    if (!Object.hasOwn(enUS, key)) throw new RangeError(`Unknown translation key: ${key}`);
    const text = catalogs[locale][key] ?? enUS[key];
    return text.replace(/\{([a-zA-Z]+)\}/g, (_, parameter) => {
      if (!Object.hasOwn(parameters, parameter)) throw new TypeError(`Missing translation parameter: ${parameter}`);
      return String(parameters[parameter]);
    });
  }
  return Object.freeze({
    locale,
    t,
    number: (value, options = {}) => new Intl.NumberFormat(locale, options).format(value),
    dateTime: (value, options = {}) => new Intl.DateTimeFormat(locale, {
      dateStyle: 'medium', timeStyle: 'medium', ...options,
    }).format(new Date(value)),
    error: (code) => Object.hasOwn(enUS, `error.${code}`)
      ? t(`error.${code}`) : t('error.generic', { code: code || 'UNKNOWN' }),
  });
}

export function presentExecution(execution, locale = 'en-US') {
  const translator = createTranslator(locale);
  const status = ['accepted', 'verified', 'failed', 'unknown', 'partial'].includes(execution.status)
    ? execution.status : 'unknown';
  const items = execution.items?.map(item => presentExecution(item, locale)) ?? [];
  return {
    status,
    label: translator.t(`execution.${status}`),
    message: execution.error ? translator.error(execution.error.code) : null,
    evidence: execution.verification ?? null,
    warnings: execution.warnings ?? [],
    items,
    // Requested values never replace an observed device state in the presentation model.
    requiresRefresh: status === 'unknown' || status === 'accepted' || execution.error?.code === 'REQUEST_RETIRED'
      || items.some(item => item.requiresRefresh),
  };
}

export function presentObservation(observation, locale = 'en-US', property = '') {
  const translator = createTranslator(locale);
  const value = observation?.value;
  let label = translator.t('device.unknown');
  if (typeof value === 'boolean') {
    label = property === 'power.state' ? translator.t(value ? 'power.on' : 'power.off')
      : translator.t(`boolean.${value}`);
  } else if (typeof value === 'number') {
    label = translator.number(value);
  } else if (value != null) {
    const key = `${property}.${value}`;
    label = Object.hasOwn(enUS, key) ? translator.t(key) : String(value);
  }
  return {
    value: value ?? null,
    label,
    stale: observation?.stale ?? true,
    receivedAt: observation?.received_at ?? null,
    observedAt: observation?.observed_at ?? null,
  };
}
