'use client';

import { type ComponentProps, type ReactNode, useTransition } from 'react';
import { toast } from 'sonner';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from '@/components/ui/alert-dialog';
import { LoadingSwap } from '@/components/ui/animated/swapper';
import { Button } from '@/components/ui/button';

export function ActionButton({
  action,
  disabled,
  requireAreYouSure = false,
  areYouSureDescription = 'این عمل غیرقابل بازگشت است.',
  areYouSureButton = 'باشه.',
  ...props
}: ComponentProps<typeof Button> & {
  action: () =>
    | Promise<{ error: boolean; message?: string }>
    | Promise<void>
    | void;
  requireAreYouSure?: boolean;
  areYouSureDescription?: ReactNode;
  areYouSureButton?: ReactNode;
}) {
  const [isLoading, startTransition] = useTransition();

  function performAction() {
    startTransition(async () => {
      try {
        const data = await action();

        if (data?.error) {
          toast.error(data.message ?? 'Error');
        }
      } catch (error) {
        console.error(error);
      }
    });
  }

  if (requireAreYouSure) {
    return (
      <AlertDialog open={isLoading ? true : undefined}>
        <AlertDialogTrigger render={<Button {...props} />} />
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>از این کار اطمینان دارید؟</AlertDialogTitle>
            <AlertDialogDescription>
              {areYouSureDescription}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>بازگشت</AlertDialogCancel>
            <AlertDialogAction
              disabled={isLoading || disabled}
              onClick={performAction}
            >
              <LoadingSwap isLoading={isLoading}>
                {areYouSureButton}
              </LoadingSwap>
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    );
  }

  return (
    <Button
      {...props}
      disabled={disabled || isLoading}
      onClick={(e) => {
        performAction();
        props.onClick?.(e);
      }}
    >
      <LoadingSwap
        className="inline-flex items-center gap-2"
        isLoading={isLoading}
      >
        {props.children}
      </LoadingSwap>
    </Button>
  );
}
