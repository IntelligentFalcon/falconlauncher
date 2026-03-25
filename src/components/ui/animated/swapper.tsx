import type { ClassValue } from 'clsx';
import { Spinner } from '@/components/ui/spinner';
import { cn } from '@/lib/utils';

export function ToggleSwap({
  render,
  renderAlternative,
  doSwap,
  className,
  containerClassName,
}: {
  render: React.ReactNode;
  renderAlternative: React.ReactNode;
  doSwap: boolean;
  className?: string;
  containerClassName?: ClassValue;
}) {
  return (
    <div
      className={cn(
        'grid grid-cols-1 items-center justify-items-center',
        containerClassName,
      )}
    >
      <div
        className={cn(
          'col-start-1 col-end-2 row-start-1 row-end-2 w-full transition-all',
          doSwap
            ? 'pointer-events-none scale-50 opacity-0'
            : 'scale-100 opacity-100',
          className,
        )}
      >
        {render}
      </div>
      <div
        className={cn(
          'col-start-1 col-end-2 row-start-1 row-end-2 transition-all',
          doSwap
            ? 'scale-100 opacity-100'
            : 'pointer-events-none scale-50 opacity-0',
          className,
        )}
      >
        {renderAlternative}
      </div>
    </div>
  );
}

export function LoadingSwap({
  children,
  isLoading,
  className,
}: {
  children: React.ReactNode;
  isLoading: boolean;
  className?: string;
}) {
  return (
    <ToggleSwap
      className={className}
      doSwap={isLoading}
      render={children}
      renderAlternative={<Spinner />}
    />
  );
}
