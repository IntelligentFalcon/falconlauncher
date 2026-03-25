'use client';

import {
  Loading01Icon,
  Loading02Icon,
  Loading03Icon,
} from '@hugeicons/core-free-icons';
import { HugeiconsIcon } from '@hugeicons/react';

import { cn } from '@/lib/utils';

function Spinner({ className, ...props }: React.ComponentProps<'svg'>) {
  return (
    <HugeiconsIcon
      icon={Loading03Icon}
      aria-label="Loading"
      className={cn('size-4 animate-spin', className)}
      role="status"
      {...props}
      strokeWidth={2}
    />
  );
}

export { Spinner };
