import { t, i18n } from '../i18n';
import { el, icon, iconButton, button, switchRow, group } from '../ui';
import { ModalController } from './ModalController';

export interface SectionSettings {
  id: string;
  name: string;
  pageStyle: string;
  columnCount?: number;
  distributionMode?: 'Sequential' | 'Balanced';
}

export interface SectionSettingsDialogCallbacks {
  onClose: () => void;
  onSave: (settings: SectionSettings) => void;
}

/**
 * Dialog for editing section properties including continuous flow column layout.
 * Phase 6: Adds UI controls for column_count and distribution_mode.
 */
export class SectionSettingsDialog {
  public element: HTMLElement;
  private callbacks: SectionSettingsDialogCallbacks;
  private isVisible = false;
  private modal: ModalController;
  private currentSettings: SectionSettings | null = null;

  constructor(callbacks: SectionSettingsDialogCallbacks) {
    this.callbacks = callbacks;
    this.element = el('div', 'tok-overlay tok-section-settings-overlay');

    this.modal = new ModalController(this.element, () => this.hide(), { closeOnBackdrop: true });

    i18n.onChange(() => {
      if (this.isVisible) this.render();
    });
  }

  public show(settings: SectionSettings): void {
    this.currentSettings = { ...settings };
    this.isVisible = true;
    this.element.classList.add('tok-open');
    this.render();
    this.modal.opened();
  }

  public hide(): void {
    if (!this.isVisible) return;
    this.isVisible = false;
    this.element.classList.remove('tok-open');
    this.modal.closed();
    this.callbacks.onClose();
  }

  private render(): void {
    const focusKey = this.modal.captureFocus();
    this.renderContent();
    this.modal.afterRender(focusKey);
  }

  private renderContent(): void {
    if (!this.currentSettings) return;

    this.element.innerHTML = '';
    this.element.dir = i18n.getDirection();

    const card = el('div', 'tok-dialog tok-section-settings-card');
    card.style.width = '540px';
    card.style.maxHeight = '88vh';

    // Header
    const head = el('div', 'tok-dialog-head');
    const headText = el('div', 'tok-dialog-head-text');
    headText.appendChild(el('h2', undefined, { 'data-modal-title': '' }, t('sectionSettingsTitle')));
    head.appendChild(headText);
    head.appendChild(iconButton('close', t('close'), () => this.hide(), { attrs: { 'data-focus-key': 'x' } }));
    card.appendChild(head);

    // Scrollable Body
    const body = el('div', 'tok-dialog-body', {
      style: 'flex:1;overflow-y:auto;padding:24px 28px;display:flex;flex-direction:column;gap:20px;'
    });

    // Section Name
    const nameGroup = el('div', undefined, { style: 'display:flex;flex-direction:column;gap:8px;' });
    const nameLabel = el('label', undefined, { style: 'font-size:13px;font-weight:600;' }, t('sectionName'));
    const nameId = `tok-section-name-${Math.random().toString(36).slice(2, 9)}`;
    nameLabel.setAttribute('for', nameId);
    const nameInput = el('input', 'tok-input', {
      type: 'text',
      id: nameId,
      value: this.currentSettings.name,
      'data-focus-key': 'name'
    });
    nameInput.addEventListener('input', () => {
      if (this.currentSettings) {
        this.currentSettings.name = nameInput.value;
      }
    });
    nameGroup.appendChild(nameLabel);
    nameGroup.appendChild(nameInput);
    body.appendChild(nameGroup);

    // Page Style
    const styleGroup = el('div', undefined, { style: 'display:flex;flex-direction:column;gap:8px;' });
    const styleLabel = el('label', undefined, { style: 'font-size:13px;font-weight:600;' }, t('sectionPageStyle'));
    const styleId = `tok-section-style-${Math.random().toString(36).slice(2, 9)}`;
    styleLabel.setAttribute('for', styleId);
    const styleSelect = el('select', 'tok-select', {
      id: styleId,
      'data-focus-key': 'style'
    }) as HTMLSelectElement;
    
    const pageStyles = ['Standard', 'Letter', 'A4', 'Wide'];
    for (const style of pageStyles) {
      const opt = el('option', undefined, { value: style }, style);
      if (style === this.currentSettings.pageStyle) {
        opt.setAttribute('selected', '');
      }
      styleSelect.appendChild(opt);
    }
    
    styleSelect.addEventListener('change', () => {
      if (this.currentSettings) {
        this.currentSettings.pageStyle = styleSelect.value;
      }
    });
    styleGroup.appendChild(styleLabel);
    styleGroup.appendChild(styleSelect);
    body.appendChild(styleGroup);

    // Column Layout Section
    const columnCard = el('div', 'tok-card', {
      style: 'padding:18px 20px;background:var(--tok-bg-surface-2);border-radius:8px;display:flex;flex-direction:column;gap:16px;'
    });
    columnCard.appendChild(el('h3', undefined, { style: 'font-size:14px;font-weight:700;margin:0;' }, t('columnLayout')));

    // Column Count
    const columnCountRow = el('div', undefined, { style: 'display:flex;align-items:center;justify-content:space-between;' });
    columnCountRow.appendChild(el('span', undefined, { style: 'font-size:13px;' }, t('columnCount')));
    
    const columnCountControl = el('div', undefined, { style: 'display:flex;align-items:center;gap:8px;' });
    const columnCountValue = el('span', undefined, {
      style: 'min-width:24px;text-align:center;font-size:14px;font-weight:600;',
      'data-column-count': ''
    }, String(this.currentSettings.columnCount || 1));
    
    const decrementBtn = iconButton('minus', t('decrease'), () => {
      if (this.currentSettings && (this.currentSettings.columnCount || 1) > 1) {
        this.currentSettings.columnCount = (this.currentSettings.columnCount || 1) - 1;
        columnCountValue.textContent = String(this.currentSettings.columnCount);
        this.updateFlowModeHint();
      }
    }, { size: 16, attrs: { 'data-focus-key': 'col-dec' } });
    
    const incrementBtn = iconButton('plus', t('increase'), () => {
      if (this.currentSettings) {
        this.currentSettings.columnCount = (this.currentSettings.columnCount || 1) + 1;
        if (this.currentSettings.columnCount > 3) {
          this.currentSettings.columnCount = 3;
        }
        columnCountValue.textContent = String(this.currentSettings.columnCount);
        this.updateFlowModeHint();
      }
    }, { size: 16, attrs: { 'data-focus-key': 'col-inc' } });
    
    columnCountControl.appendChild(decrementBtn);
    columnCountControl.appendChild(columnCountValue);
    columnCountControl.appendChild(incrementBtn);
    columnCountRow.appendChild(columnCountControl);
    columnCard.appendChild(columnCountRow);

    // Flow Mode Hint
    const flowModeHint = el('p', undefined, {
      style: 'font-size:12px;color:var(--tok-text-muted);margin:0;line-height:1.4;',
      'data-flow-hint': ''
    });
    this.updateFlowModeHintElement(flowModeHint);
    columnCard.appendChild(flowModeHint);

    // Distribution Mode (only shown for multi-column)
    const distributionRow = el('div', undefined, {
      style: 'display:flex;flex-direction:column;gap:8px;',
      'data-distribution-row': ''
    });
    distributionRow.appendChild(el('span', undefined, { style: 'font-size:13px;font-weight:600;' }, t('distributionMode')));
    
    const distributionRadios = el('div', undefined, { style: 'display:flex;flex-direction:column;gap:6px;padding-inline-start:8px;' });
    
    const balancedId = `tok-dist-bal-${Math.random().toString(36).slice(2, 9)}`;
    const balancedRadio = el('input', undefined, {
      type: 'radio',
      name: 'distribution',
      id: balancedId,
      value: 'Balanced',
      'data-focus-key': 'dist-bal'
    }) as HTMLInputElement;
    if ((this.currentSettings.distributionMode || 'Balanced') === 'Balanced') {
      balancedRadio.checked = true;
    }
    balancedRadio.addEventListener('change', () => {
      if (this.currentSettings && balancedRadio.checked) {
        this.currentSettings.distributionMode = 'Balanced';
      }
    });
    const balancedLabel = el('label', undefined, {
      for: balancedId,
      style: 'display:flex;align-items:center;gap:8px;cursor:pointer;'
    });
    balancedLabel.appendChild(balancedRadio);
    balancedLabel.appendChild(el('span', undefined, { style: 'font-size:13px;' }, t('balanced')));
    distributionRadios.appendChild(balancedLabel);
    
    const sequentialId = `tok-dist-seq-${Math.random().toString(36).slice(2, 9)}`;
    const sequentialRadio = el('input', undefined, {
      type: 'radio',
      name: 'distribution',
      id: sequentialId,
      value: 'Sequential',
      'data-focus-key': 'dist-seq'
    }) as HTMLInputElement;
    if (this.currentSettings.distributionMode === 'Sequential') {
      sequentialRadio.checked = true;
    }
    sequentialRadio.addEventListener('change', () => {
      if (this.currentSettings && sequentialRadio.checked) {
        this.currentSettings.distributionMode = 'Sequential';
      }
    });
    const sequentialLabel = el('label', undefined, {
      for: sequentialId,
      style: 'display:flex;align-items:center;gap:8px;cursor:pointer;'
    });
    sequentialLabel.appendChild(sequentialRadio);
    sequentialLabel.appendChild(el('span', undefined, { style: 'font-size:13px;' }, t('sequential')));
    distributionRadios.appendChild(sequentialLabel);
    
    distributionRow.appendChild(distributionRadios);
    distributionRow.appendChild(el('p', undefined, {
      style: 'font-size:11px;color:var(--tok-text-muted);margin:0;'
    }, t('distributionHint')));
    
    if ((this.currentSettings.columnCount || 1) > 1) {
      columnCard.appendChild(distributionRow);
    }
    
    body.appendChild(columnCard);
    card.appendChild(body);

    // Footer with buttons
    const foot = el('div', 'tok-dialog-foot');
    
    foot.appendChild(button(t('cancel'), {
      className: 'tok-btn',
      attrs: { 'data-focus-key': 'cancel' },
      onClick: () => this.hide()
    }));

    foot.appendChild(el('span', 'tok-grow'));

    foot.appendChild(button(t('apply'), {
      className: 'tok-btn tok-btn-primary',
      icon: 'check',
      attrs: { 'data-focus-key': 'save' },
      onClick: () => this.handleSave()
    }));
    
    card.appendChild(foot);
    this.element.appendChild(card);
  }

  private updateFlowModeHint(): void {
    const hintEl = this.element.querySelector('[data-flow-hint]') as HTMLElement | null;
    if (hintEl) {
      this.updateFlowModeHintElement(hintEl);
    }
    
    // Show/hide distribution options
    const distRow = this.element.querySelector('[data-distribution-row]') as HTMLElement | null;
    if (distRow && this.currentSettings) {
      if ((this.currentSettings.columnCount || 1) > 1) {
        distRow.style.display = 'flex';
      } else {
        distRow.style.display = 'none';
      }
    }
  }

  private updateFlowModeHintElement(el: HTMLElement): void {
    if (!this.currentSettings) return;
    const colCount = this.currentSettings.columnCount || 1;
    if (colCount === 1) {
      el.textContent = t('flowModeSingle');
    } else {
      el.textContent = t('flowModeContinuous');
    }
  }

  private handleSave(): void {
    if (!this.currentSettings) return;
    
    // Clean up settings: if column_count is 1, remove it and distribution_mode
    if (this.currentSettings.columnCount === 1) {
      delete this.currentSettings.columnCount;
      delete this.currentSettings.distributionMode;
    }
    
    this.callbacks.onSave({ ...this.currentSettings });
    this.hide();
  }
}
