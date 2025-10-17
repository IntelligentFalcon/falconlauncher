import { Command, CommandError } from '@/types';
import { invoke } from '@tauri-apps/api/core';
import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function command<T extends Command<any, any>>(
  command: string,
  args: Parameters<T>['0']
): Promise<ReturnType<T>> {
  return new Promise((resolve, reject) => {
    invoke<ReturnType<T>>(command, args)
      .then((result) => {
        resolve(result);
      })
      .catch((error: CommandError) => {
        reject(error);
      });
  });
}
