import { InvokeError, Invokes } from '@/invokes';
import { backend } from '@/lib/utils';
import {
  QueryKey,
  useMutation,
  UseMutationOptions,
  useQuery,
  UseQueryOptions,
} from '@tanstack/react-query';
import { toast } from 'sonner'; // Replace with your toast library (e.g., @/components/ui/use-toast)
import errorsMap from '@/messages/errors.json';

// --- Generic Error Handler ---
// Extracts the error code, maps it, and decides whether to toast or show a dialog.
const handleBackendError = (error: any) => {
  // Adjust this based on how your Rust backend formats the error object
  const errorCode = error?.code || error?.message || 'UNKNOWN_ERROR';

  // @ts-ignore - Assuming errors.json is a key-value map of code -> message
  const displayMessage = errorsMap[errorCode] || error?.message || 'An unexpected error occurred.';

  // TODO: We need to define which error codes trigger dialogs
  const requiresDialog = ['GAME_ERROR', 'SOME_OTHER_ERROR'].includes(errorCode);

  if (requiresDialog) {
    // 🛑 STOPPING HERE FOR DIALOG LOGIC
    // We will implement the dialog trigger here based on your instructions.
    console.warn('Dialog required for:', errorCode, displayMessage);
  } else {
    // Standard toast fallback
    toast.error(displayMessage);
  }
};

export function useBackend<
    Invoke extends keyof Invokes,
    TData = Invokes[Invoke]['returns'],
>({
    name,
    args,
    ...params
  }: Omit<
    UseQueryOptions<Invokes[Invoke]['returns'], unknown, TData>,
    'queryFn' | 'queryKey'
> & { name: Invoke; args?: Invokes[Invoke]['args']; queryKey?: QueryKey }) {
  const query = useQuery({
    queryKey: name.split('_'),
    queryFn: async () => {
      try {
        return await backend(name, args);
      } catch (error) {
        handleBackendError(error);
        throw error; // Rethrow so React Query knows it failed
      }
    },
    ...params,
  });

  return query;
}

type TVarsType<
    Args extends Invokes[keyof Invokes]['args'],
    TArgs extends Partial<Record<string, unknown>>,
> = keyof Omit<Args, keyof TArgs> extends never
    ? void // بدون پارامتر
    : Omit<Args, keyof TArgs>; // همان تعریف قبلی

export function useBackendMutation<
    Invoke extends keyof Invokes,
    TArgs extends Partial<Invokes[Invoke]['args']> = {},
>({
    name,
    args,
    ...params
  }: Omit<
    UseMutationOptions<
        Invokes[Invoke]['returns'], // TData
        InvokeError<Invokes[Invoke]['custom_error']>, // TError
        TVarsType<Invokes[Invoke]['args'], TArgs> // TVariables
    >,
    'mutationFn'
> & { name: Invoke; args?: TArgs }) {
  type TVars = TVarsType<Invokes[Invoke]['args'], TArgs>;

  const mutation = useMutation<
      Invokes[Invoke]['returns'], // TData
      InvokeError<Invokes[Invoke]['custom_error']>, // TError
      TVars // TVariables
  >({
    mutationKey: name.split('_'),
    mutationFn: async (variables: TVars) => {
      try {
        return await backend(name, { ...args, ...variables } as Invokes[Invoke]['args']);
      } catch (error) {
        handleBackendError(error);
        throw error; // Rethrow so React Query knows it failed
      }
    },
    ...params,
  });

  return mutation;
}