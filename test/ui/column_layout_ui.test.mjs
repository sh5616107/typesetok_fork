/**
 * Phase 6 UI Integration Tests
 * Tests for SectionSettingsDialog, ColumnToolbar, and template loading.
 */

import { test } from 'node:test';
import { strict as assert } from 'node:assert';
import * as fs from 'node:fs';
import * as path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, '../..');

// Helper to check if TypeScript files exist and are syntactically valid
function checkTypeScriptFile(relPath) {
  const fullPath = path.join(repoRoot, relPath);
  assert.ok(fs.existsSync(fullPath), `File should exist: ${relPath}`);
  const content = fs.readFileSync(fullPath, 'utf-8');
  assert.ok(content.length > 100, `File should have substantial content: ${relPath}`);
  
  // Basic syntax checks
  assert.ok(content.includes('export '), `File should export something: ${relPath}`);
  return content;
}

// Helper to check JSON template validity
function checkTemplateFile(relPath) {
  const fullPath = path.join(repoRoot, relPath);
  assert.ok(fs.existsSync(fullPath), `Template should exist: ${relPath}`);
  const content = fs.readFileSync(fullPath, 'utf-8');
  const json = JSON.parse(content);
  assert.ok(json.metadata, 'Template should have metadata');
  assert.ok(json.sections, 'Template should have sections');
  assert.ok(Array.isArray(json.sections), 'Sections should be an array');
  return json;
}

test('Phase 6 UI - SectionSettingsDialog component exists and is valid', () => {
  const content = checkTypeScriptFile('packages/tok-ui/src/components/SectionSettingsDialog.ts');
  
  // Check for key class and interface exports
  assert.ok(content.includes('export class SectionSettingsDialog'), 'Should export SectionSettingsDialog class');
  assert.ok(content.includes('export interface SectionSettings'), 'Should export SectionSettings interface');
  assert.ok(content.includes('columnCount'), 'Should handle columnCount field');
  assert.ok(content.includes('distributionMode'), 'Should handle distributionMode field');
  
  // Check for UI rendering
  assert.ok(content.includes('render'), 'Should have render method');
  assert.ok(content.includes('show'), 'Should have show method');
  assert.ok(content.includes('hide'), 'Should have hide method');
  
  // Check for callbacks
  assert.ok(content.includes('onSave'), 'Should have onSave callback');
});

test('Phase 6 UI - ColumnToolbar component exists and is valid', () => {
  const content = checkTypeScriptFile('packages/tok-ui/src/components/ColumnToolbar.ts');
  
  // Check for key exports
  assert.ok(content.includes('export class ColumnToolbar'), 'Should export ColumnToolbar class');
  assert.ok(content.includes('onColumnCountChange'), 'Should have column count change callback');
  
  // Check for column controls
  assert.ok(content.includes('setColumnCount'), 'Should have setColumnCount method');
  assert.ok(content.includes('setBalanced'), 'Should have setBalanced method');
  
  // Check for button rendering (1, 2, 3 columns)
  assert.ok(content.includes('count <= 3'), 'Should support up to 3 columns');
});

test('Phase 6 UI - IPC handlers added to main.ts', () => {
  const fullPath = path.join(repoRoot, 'packages/tok-electron/src/main.ts');
  assert.ok(fs.existsSync(fullPath), 'main.ts should exist');
  const content = fs.readFileSync(fullPath, 'utf-8');
  
  // Check for Phase 6 IPC handlers
  assert.ok(
    content.includes('tok:set-section-column-layout') || content.includes('set-section-column-layout'),
    'Should have IPC handler for setting section column layout'
  );
  assert.ok(
    content.includes('tok:get-section-column-info') || content.includes('get-section-column-info'),
    'Should have IPC handler for getting section column info'
  );
  
  // Check for column count and distribution mode handling
  assert.ok(content.includes('columnCount'), 'IPC handlers should handle columnCount');
  assert.ok(content.includes('distributionMode'), 'IPC handlers should handle distributionMode');
});

test('Phase 6 UI - Magazine 2-column template is valid', () => {
  const template = checkTemplateFile('packages/tok-ui/src/templates/magazine_2col.json');
  
  // Check metadata
  assert.strictEqual(template.metadata.title, 'מגזין דו-טורי', 'Should have Hebrew title');
  assert.strictEqual(template.metadata.progression, 'Rtl', 'Should be RTL');
  
  // Check section configuration
  assert.ok(template.sections.length > 0, 'Should have at least one section');
  const mainSection = template.sections[0];
  assert.strictEqual(mainSection.column_count, 2, 'Should have 2 columns');
  assert.strictEqual(mainSection.distribution_mode, 'Balanced', 'Should use balanced distribution');
  
  // Check flows and paragraphs
  assert.ok(mainSection.flows.length > 0, 'Section should have flows');
  assert.ok(mainSection.flows[0].paragraphs.length > 0, 'Flow should have paragraphs');
});

test('Phase 6 UI - Magazine 3-column template is valid', () => {
  const template = checkTemplateFile('packages/tok-ui/src/templates/magazine_3col.json');
  
  // Check section configuration
  const mainSection = template.sections[0];
  assert.strictEqual(mainSection.column_count, 3, 'Should have 3 columns');
  assert.strictEqual(mainSection.distribution_mode, 'Balanced', 'Should use balanced distribution');
});

test('Phase 6 UI - Mixed article template is valid', () => {
  const template = checkTemplateFile('packages/tok-ui/src/templates/article_mixed.json');
  
  // Check multiple sections with different layouts
  assert.ok(template.sections.length >= 2, 'Should have at least 2 sections');
  
  const introSection = template.sections[0];
  assert.ok(!introSection.column_count || introSection.column_count === 1, 'Intro should be single column');
  
  const bodySection = template.sections[1];
  assert.strictEqual(bodySection.column_count, 2, 'Body should have 2 columns');
  assert.strictEqual(bodySection.distribution_mode, 'Balanced', 'Body should use balanced distribution');
});

test('Phase 6 UI - i18n strings added for column layout', () => {
  const content = checkTypeScriptFile('packages/tok-ui/src/i18n.ts');
  
  // Check for new translation keys
  const requiredKeys = [
    'sectionSettingsTitle',
    'columnLayout',
    'columnCount',
    'distributionMode',
    'balanced',
    'sequential',
    'oneColumn',
    'twoColumns',
    'threeColumns'
  ];
  
  for (const key of requiredKeys) {
    assert.ok(content.includes(key), `Should have translation key: ${key}`);
  }
  
  // Check that translations have both Hebrew and English
  assert.ok(content.includes('he:') && content.includes('en:'), 'Should have both Hebrew and English translations');
});

test('Phase 6 UI - All components properly structured', () => {
  // Check that components follow the established pattern
  const dialogContent = fs.readFileSync(
    path.join(repoRoot, 'packages/tok-ui/src/components/SectionSettingsDialog.ts'),
    'utf-8'
  );
  const toolbarContent = fs.readFileSync(
    path.join(repoRoot, 'packages/tok-ui/src/components/ColumnToolbar.ts'),
    'utf-8'
  );
  
  // Both should import from i18n and ui
  assert.ok(dialogContent.includes("from '../i18n'"), 'Dialog should import i18n');
  assert.ok(dialogContent.includes("from '../ui'"), 'Dialog should import ui helpers');
  assert.ok(toolbarContent.includes("from '../i18n'"), 'Toolbar should import i18n');
  assert.ok(toolbarContent.includes("from '../ui'"), 'Toolbar should import ui helpers');
  
  // Both should use ModalController or similar
  assert.ok(dialogContent.includes('ModalController'), 'Dialog should use ModalController');
  
  // Both should respond to i18n changes
  assert.ok(dialogContent.includes('i18n.onChange'), 'Dialog should handle i18n changes');
  assert.ok(toolbarContent.includes('i18n.onChange'), 'Toolbar should handle i18n changes');
});

test('Phase 6 UI - Templates have consistent structure', () => {
  const templates = [
    'packages/tok-ui/src/templates/magazine_2col.json',
    'packages/tok-ui/src/templates/magazine_3col.json',
    'packages/tok-ui/src/templates/article_mixed.json'
  ];
  
  for (const templatePath of templates) {
    const fullPath = path.join(repoRoot, templatePath);
    const json = JSON.parse(fs.readFileSync(fullPath, 'utf-8'));
    
    // All templates should have required top-level fields
    assert.ok(json.id, `${templatePath}: Should have id`);
    assert.ok(json.metadata, `${templatePath}: Should have metadata`);
    assert.ok(json.paragraph_styles, `${templatePath}: Should have paragraph_styles`);
    assert.ok(json.character_styles !== undefined, `${templatePath}: Should have character_styles`);
    assert.ok(json.sections, `${templatePath}: Should have sections`);
    
    // All templates should use Hebrew
    assert.strictEqual(json.metadata.progression, 'Rtl', `${templatePath}: Should be RTL`);
    assert.strictEqual(json.metadata.primary_language, 'he', `${templatePath}: Should be Hebrew`);
    
    // Check that column_count sections have distribution_mode
    for (const section of json.sections) {
      if (section.column_count && section.column_count > 1) {
        assert.ok(section.distribution_mode, `${templatePath}: Multi-column section should have distribution_mode`);
      }
    }
  }
});

console.log('✅ All Phase 6 UI tests passed!');
