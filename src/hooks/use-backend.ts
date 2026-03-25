import { InvokeError, Invokes } from '@/invokes';
import { backend } from '@/lib/utils';
import {
  QueryKey,
  useMutation,
  UseMutationOptions,
  useQuery,
  UseQueryOptions,
} from '@tanstack/react-query';

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
    queryFn: () => backend(name, args),
    ...params,
  });

  return query;
}

export function useBackendMutation<
  Invoke extends keyof Invokes,
  TData = unknown,
  TVariables = void,
  TContext = unknown,
>({
  name,
  args,
  ...params
}: Omit<
  UseMutationOptions<
    TData,
    InvokeError<Invokes[Invoke]['custom_error']>,
    TVariables,
    TContext
  >,
  'mutationFn'
> & { name: Invoke; args?: Invokes[Invoke]['args'] }) {
  const mutation = useMutation<
    TData,
    InvokeError<Invokes[Invoke]['custom_error']>,
    TVariables,
    TContext
  >({
    mutationKey: name.split('_'),
    mutationFn: () => backend(name, args),
    ...params,
  });

  return mutation;
}
