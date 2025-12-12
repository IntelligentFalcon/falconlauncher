import { InvokeError, Invokes } from '@/invokes';
import { invoke } from '@tauri-apps/api/core';
import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function command<T extends keyof Invokes>(
  name: T,
  args?: Invokes[T]['args']
) {
  return new Promise((resolve, reject) => {
    invoke<Invokes[T]['returns']>(name, args)
      .then((result) => {
        resolve(result);
      })
      .catch((error: InvokeError<Invokes[T]['custom_error']>) => {
        reject(error);
      });
  });
}
