import { useEffect, useId, useRef, useState, type FormEvent, type ReactNode } from 'react';
import { CheckCircle2, Gem, Loader2, X } from 'lucide-react';
import { Button } from '@sdkwork/ui-pc-react';
import { SdkworkOrderCheckoutDialog } from '@sdkwork/order-pc-checkout';
import { SdkworkPointsRechargeDialog } from '@sdkwork/order-pc-recharge';
import type {
  SdkworkSubscriptionCatalogCheckoutModalProps,
  SdkworkSubscriptionCatalogModalProps,
} from '@sdkwork/membership-pc-subscription/catalog';
import { getAgentsTokenPlanRuntime } from './runtime';

export function AgentsTokenPlanCheckoutModal({
  isOpen,
  onClose,
  onPaymentCompleted,
  onPaymentStatus,
  onPurchase,
  plan,
}: SdkworkSubscriptionCatalogCheckoutModalProps) {
  return (
    <SdkworkOrderCheckoutDialog
      copy={{
        activationDescription: '支付确认后，会员权益将自动生效。',
        activationTitle: '自动开通',
        close: '关闭',
        completed: '支付完成',
        creatingPayment: '正在创建支付订单...',
        paymentUnavailable: '暂时无法支付',
        paymentUnavailableDescription: '支付订单创建失败，请稍后重试。',
        payByQr: '扫码支付',
        price: '价格',
        retry: '重试',
        scanPrompt: '请扫描二维码完成支付',
        secureDescription: '订单和支付由 SDKWork Order 安全处理。',
        secureTitle: '安全结算',
        selectedItem: '已选会员方案',
      }}
      driver={{
        createPayment: onPurchase,
        getPaymentStatus: onPaymentStatus
          ? (payment) => payment.orderId
            ? onPaymentStatus(payment.orderId)
            : Promise.resolve({ ...payment, status: 'failed' })
          : undefined,
        onPaymentCompleted,
      }}
      isOpen={isOpen}
      onClose={onClose}
      summary={plan ? {
        id: plan.id,
        name: plan.name,
        originalPriceLabel: plan.originalPrice,
        periodLabel: plan.packagePeriodLabel,
        priceLabel: plan.priceLabel,
      } : null}
    />
  );
}

export function AgentsTokenPlanPointsPurchaseModal({
  currentPoints,
  isOpen,
  onClose,
}: SdkworkSubscriptionCatalogModalProps) {
  const service = getAgentsTokenPlanRuntime().pointsRechargeService;
  if (!service) {
    return <AgentsTokenPlanInfoModal currentPoints={currentPoints} isOpen={isOpen} onClose={onClose} />;
  }
  return (
    <SdkworkPointsRechargeDialog
      copy={{
        account: 'SDKWork Agents',
        agreement: '支付前请阅读并同意算力积分充值服务协议。',
        agreementAccepted: '您已同意算力积分充值服务协议。',
        agreementRequired: '请先同意算力积分充值服务协议。',
        close: '关闭',
        completed: '支付完成，算力积分已到账。',
        confirmPayment: '同意并支付',
        creatingPayment: '正在生成支付二维码...',
        emptyPackages: '暂无可用充值套餐。',
        loadFailed: '充值套餐加载失败。',
        loadingPackages: '正在加载充值套餐...',
        myPoints: '我的算力积分',
        notice: '算力积分不可转赠或提现，到账及有效期以平台规则为准。',
        paymentUnavailable: '暂时无法支付',
        paymentUnavailableDescription: '支付二维码生成失败，请稍后重试。',
        pointsUnit: '算力积分',
        retry: '重新加载',
        scanPrompt: '请扫码完成支付',
        title: '购买算力积分',
      }}
      currentPoints={currentPoints}
      isOpen={isOpen}
      onClose={onClose}
      service={service}
    />
  );
}

export function AgentsTokenPlanPointsDetailsModal(props: SdkworkSubscriptionCatalogModalProps) {
  return <AgentsTokenPlanInfoModal {...props} />;
}

function AgentsTokenPlanInfoModal({
  currentPoints,
  isOpen,
  onClose,
}: SdkworkSubscriptionCatalogModalProps) {
  useEscapeToClose(isOpen, onClose);
  if (!isOpen) return null;
  return (
    <TokenPlanDialog onClose={onClose} title="算力积分">
      <div className="flex items-center justify-between rounded-lg border border-white/10 bg-black/20 px-4 py-4">
        <span className="text-sm text-zinc-400">当前余额</span>
        <span className="text-2xl font-semibold text-cyan-300">{currentPoints ?? 0}</span>
      </div>
      <p className="text-sm leading-6 text-zinc-400">积分可用于 SDKWork Agents 中支持的模型调用和智能体能力。</p>
      <div className="flex justify-end"><Button onClick={onClose}>完成</Button></div>
    </TokenPlanDialog>
  );
}

export function AgentsTokenPlanRedeemModal({ isOpen, onClose }: SdkworkSubscriptionCatalogModalProps) {
  const inputId = useId();
  const inputRef = useRef<HTMLInputElement>(null);
  const [code, setCode] = useState('');
  const [error, setError] = useState('');
  const [grantAmount, setGrantAmount] = useState<number | null>(null);
  const [submitting, setSubmitting] = useState(false);
  useEscapeToClose(isOpen && !submitting, onClose);

  useEffect(() => {
    if (!isOpen) {
      setCode('');
      setError('');
      setGrantAmount(null);
      setSubmitting(false);
      return;
    }
    const frame = window.requestAnimationFrame(() => inputRef.current?.focus());
    return () => window.cancelAnimationFrame(frame);
  }, [isOpen]);

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const service = getAgentsTokenPlanRuntime().couponRechargeService;
    if (!code.trim()) {
      setError('请输入兑换码。');
      inputRef.current?.focus();
      return;
    }
    if (!service) {
      setError('当前运行环境尚未配置兑换服务。');
      return;
    }
    setSubmitting(true);
    setError('');
    try {
      const result = await service.redeem(code.trim());
      setGrantAmount(
        result.benefitKind === 'token_bank_credit' ? result.grantAmount : result.totalQuota,
      );
      setCode('');
    } catch (reason) {
      setError(reason instanceof Error && reason.message.trim() ? reason.message : '兑换失败，请检查兑换码后重试。');
    } finally {
      setSubmitting(false);
    }
  }

  if (!isOpen) return null;
  return (
    <TokenPlanDialog onClose={onClose} title="会员与积分兑换">
      {grantAmount !== null ? (
        <div className="space-y-5 text-center">
          <CheckCircle2 aria-hidden="true" className="mx-auto h-12 w-12 text-emerald-400" />
          <p className="text-sm text-zinc-300">兑换成功，已到账 {grantAmount} 算力积分。</p>
          <Button onClick={onClose}>完成</Button>
        </div>
      ) : (
        <form className="space-y-4" onSubmit={submit}>
          <label className="block text-sm font-medium text-zinc-200" htmlFor={inputId}>兑换码</label>
          <input
            autoComplete="off"
            className="h-11 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 text-sm text-white outline-none focus:border-cyan-500 focus:ring-2 focus:ring-cyan-500/20"
            disabled={submitting}
            id={inputId}
            onChange={(event) => { setCode(event.target.value); setError(''); }}
            placeholder="请输入兑换码"
            ref={inputRef}
            value={code}
          />
          <div aria-live="assertive" className="min-h-6 text-sm text-rose-300">{error}</div>
          <div className="flex justify-end gap-3">
            <Button disabled={submitting} onClick={onClose} type="button" variant="ghost">取消</Button>
            <Button disabled={submitting || !code.trim()} type="submit">
              {submitting ? <Loader2 aria-hidden="true" className="mr-2 h-4 w-4 animate-spin" /> : null}
              立即兑换
            </Button>
          </div>
        </form>
      )}
    </TokenPlanDialog>
  );
}

function TokenPlanDialog({ children, onClose, title }: { children: ReactNode; onClose: () => void; title: string }) {
  return (
    <div className="fixed inset-0 z-[70] flex items-center justify-center p-4">
      <button aria-label="关闭" className="absolute inset-0 bg-black/70 backdrop-blur-sm" onClick={onClose} type="button" />
      <div aria-modal="true" className="relative w-full max-w-md rounded-lg border border-white/10 bg-[#1e1e22] shadow-2xl" role="dialog">
        <div className="flex items-center justify-between border-b border-white/10 px-5 py-4">
          <div className="flex items-center gap-3"><Gem aria-hidden="true" className="h-5 w-5 text-cyan-400" /><h2 className="text-base font-semibold text-white">{title}</h2></div>
          <button aria-label="关闭" className="rounded-md p-1.5 text-zinc-400 hover:bg-white/10 hover:text-white" onClick={onClose} type="button"><X className="h-5 w-5" /></button>
        </div>
        <div className="space-y-5 p-5">{children}</div>
      </div>
    </div>
  );
}

function useEscapeToClose(enabled: boolean, onClose: () => void) {
  useEffect(() => {
    if (!enabled) return;
    const onKeyDown = (event: KeyboardEvent) => { if (event.key === 'Escape') onClose(); };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [enabled, onClose]);
}
