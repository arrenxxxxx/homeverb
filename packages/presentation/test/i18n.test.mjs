import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { createTranslator, normalizeLocale, presentExecution, presentObservation, supportedLocales } from '../src/index.mjs';

test('catalogs have identical English keys and interpolation parameters', async () => {
  const catalogs = await Promise.all(supportedLocales.map(async locale =>
    JSON.parse(await readFile(new URL(`../locales/${locale}.json`, import.meta.url)))));
  assert.deepEqual(Object.keys(catalogs[0]).sort(), Object.keys(catalogs[1]).sort());
  for (const key of Object.keys(catalogs[0])) {
    assert.match(key, /^[a-zA-Z][a-zA-Z0-9._]*$/);
    assert.ok(catalogs.every(catalog => typeof catalog[key] === 'string' && catalog[key].length > 0));
    assert.deepEqual(catalogs[0][key].match(/\{\w+\}/g), catalogs[1][key].match(/\{\w+\}/g));
  }
});

test('only en-US and zh-CN are selected, with English fallback', () => {
  assert.equal(normalizeLocale('EnUs'), 'en-US');
  assert.equal(normalizeLocale('ZhCN'), 'zh-CN');
  assert.equal(normalizeLocale('zh_CN'), 'zh-CN');
  assert.equal(normalizeLocale('zh-TW'), 'en-US');
  assert.equal(normalizeLocale('fr-FR'), 'en-US');
  assert.notEqual(createTranslator('en-US').t('device.list'), createTranslator('zh-CN').t('device.list'));
});

test('interpolation is explicit and unknown keys fail during development', () => {
  const translator = createTranslator();
  assert.equal(translator.t('state.pendingTarget', { value: 26 }), 'Requested: 26');
  assert.throws(() => translator.t('state.pendingTarget'), /Missing translation parameter/);
  assert.throws(() => translator.t('missing'), /Unknown translation key/);
  assert.match(translator.error('NEW_ERROR'), /NEW_ERROR/);
});

test('unknown observations and uncertain outcomes do not become optimistic state', () => {
  const unknown = presentObservation(null);
  assert.equal(unknown.value, null);
  assert.equal(unknown.label, 'Unknown');
  assert.equal(presentObservation({ value: false, stale: false }).value, false);
  assert.equal(presentObservation({ value: false }, 'en-US', 'power.state').label, 'Off');
  assert.equal(presentObservation({ value: 'heat' }, 'en-US', 'climate.mode').label, 'Heat');
  assert.notEqual(presentObservation({ value: 'heat' }, 'zh-CN', 'climate.mode').label, 'Heat');
  const result = presentExecution({ status: 'unknown', data: { state: true } });
  assert.equal(result.requiresRefresh, true);
  assert.equal(result.evidence, null);
  assert.equal(result.value, undefined);
  assert.equal(presentExecution({ status: 'new-status' }).status, 'unknown');
  assert.equal(presentExecution({ status: 'failed', error: { code: 'REQUEST_RETIRED' } }).requiresRefresh, true);
});

test('composite presentation retains evidence and requests refresh for uncertain children', async () => {
  const fixture = JSON.parse(await readFile(new URL('../../../examples/command-result.storage-failure.json', import.meta.url)));
  for (const locale of supportedLocales) {
    const known = presentExecution(fixture, locale);
    assert.equal(known.requiresRefresh, false);
    assert.deepEqual(known.items[0].evidence, fixture.items[0].verification);
    assert.deepEqual(known.items[0].warnings, fixture.items[0].warnings);
    assert.equal(known.items[1].status, 'failed');
    for (const status of ['accepted', 'unknown']) {
      const composite = presentExecution({ status: 'partial', items: [
        { status: 'verified', verification: fixture.items[0].verification },
        { status: 'partial', items: [{ status }, { status: 'failed' }] },
      ] }, locale);
      assert.equal(composite.requiresRefresh, true);
      assert.equal(composite.items[1].requiresRefresh, true);
    }
  }
});
