import * as React from 'react';
import { Slot } from '@radix-ui/react-slot';
import { cva, type VariantProps } from 'class-variance-authority';

import { cn } from '@/lib/utils';

const buttonVariants = cva('az-btn tracking-normal', {
  variants: {
    variant: {
      default: 'az-btn-primary',
      secondary: 'az-btn-secondary',
      success: 'az-btn-success',
      warning: 'az-btn-warning',
      danger: 'az-btn-danger',
      ghost: 'az-btn-ghost',
    },
    size: {
      default: '',
      xs: 'az-btn-xs',
      sm: 'az-btn-sm',
      lg: 'az-btn-lg',
      xl: 'az-btn-xl',
    },
  },
  defaultVariants: {
    variant: 'default',
    size: 'default',
  },
});

function Button({
  className,
  variant,
  size,
  asChild = false,
  ...props
}: React.ComponentProps<'button'> &
  VariantProps<typeof buttonVariants> & {
    asChild?: boolean;
  }) {
  const Comp = asChild ? Slot : 'button';

  return (
    <Comp
      data-slot="button"
      className={cn(buttonVariants({ variant, size, className }))}
      {...props}
    />
  );
}

export { Button, buttonVariants };
