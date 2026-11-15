import { t, i18n } from '../i18n';
import { el, button } from '../ui';

export interface ColumnToolbarCallbacks {
  onColumnCountChange: (count: number, balanced: boolean) => void;
}

/**
 * Quick access toolbar for column layout controls.
 * Phase 6: Provides 1/2/3 column buttons and balance toggle.
 */
export class ColumnToolbar {
  public element: HTMLElement;
  private callbacks: ColumnToolbarCallbacks;
  private currentColumnCount = 1;
  private isBalanced = true;

  constructor(callbacks: ColumnToolbarCallbacks) {
    this.callbacks = callbacks;
    this.element = el('div', 'tok-column-toolbar', {
      style: 'display:flex;align-items:center;gap:8px;padding:8px 12px;background:var(--tok-bg-surface);border-bottom:1px solid var(--tok-border-subtle);'
    });
    
    this.render();
    
    i18n.onChange(() => this.render());
  }

  public setColumnCount(count: number): void {
    this.currentColumnCount = count;
    this.render();
  }

  public setBalanced(balanced: boolean): void {
    this.isBalanced = balanced;
    this.render();
  }

  public hide(): void {
    this.element.style.display = 'none';
  }

  public show(): void {
    this.element.style.display = 'flex';
  }

  private render(): void {
    this.element.innerHTML = '';
    this.element.dir = i18n.getDirection();

    // Title label
    const label = el('span', undefined, {
      style: 'font-size:12px;font-weight:600;color:var(--tok-text-secondary);'
    }, t('columnToolbarTitle'));
    this.element.appendChild(label);

    // Column count buttons
    const btnGroup = el('div', undefined, {
      style: 'display:flex;gap:4px;'
    });

    for (let count = 1; count <= 3; count++) {
      const btnLabel = count === 1 ? t('oneColumn') : count === 2 ? t('twoColumns') : t('threeColumns');
      const isActive = this.currentColumnCount === count;
      const btn = button(String(count), {
        className: isActive ? 'tok-btn tok-btn-sm tok-btn-primary' : 'tok-btn tok-btn-sm',
        attrs: {
          title: btnLabel,
          'aria-label': btnLabel,
          'aria-pressed': String(isActive)
        },
        onClick: () => this.handleColumnChange(count)
      });
      btn.style.minWidth = '40px';
      btnGroup.appendChild(btn);
    }

    this.element.appendChild(btnGroup);

    // Balance toggle (only visible for multi-column)
    if (this.currentColumnCount > 1) {
      const balanceBtn = button(t('toggleBalance'), {
        className: this.isBalanced ? 'tok-btn tok-btn-sm tok-btn-primary' : 'tok-btn tok-btn-sm',
        attrs: {
          'aria-pressed': String(this.isBalanced)
        },
        onClick: () => this.handleBalanceToggle()
      });
      this.element.appendChild(balanceBtn);
    }

    // Spacer
    this.element.appendChild(el('span', 'tok-grow'));

    // Info hint
    const hint = el('span', undefined, {
      style: 'font-size:11px;color:var(--tok-text-muted);'
    }, this.getHintText());
    this.element.appendChild(hint);
  }

  private handleColumnChange(count: number): void {
    this.currentColumnCount = count;
    this.render();
    this.callbacks.onColumnCountChange(count, this.isBalanced);
  }

  private handleBalanceToggle(): void {
    this.isBalanced = !this.isBalanced;
    this.render();
    this.callbacks.onColumnCountChange(this.currentColumnCount, this.isBalanced);
  }

  private getHintText(): string {
    if (this.currentColumnCount === 1) {
      return t('flowModeSingle');
    }
    return this.isBalanced ? t('balanced') : t('sequential');
  }
}
