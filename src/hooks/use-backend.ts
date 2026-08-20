import {
  type QueryKey,
  type UseMutationOptions,
  type UseQueryOptions,
  useMutation,
  useQuery,
} from "@tanstack/react-query";
import { sendNotification } from "@tauri-apps/plugin-notification";
import { toast } from "sonner";
import type { InvokeError, Invokes } from "@/invokes";
import { backend } from "@/lib/utils";
import { errorText } from "@/messages";
import {invoke} from "@tauri-apps/api/core";

export function useBackend<
  Invoke extends keyof Invokes,
  TData = Invokes[Invoke]["returns"],
>({
  name,
  args,
  ...params
}: Omit<
  UseQueryOptions<
    Invokes[Invoke]["returns"],
    InvokeError<Invokes[Invoke]["custom_error"]>,
    TData
  >,
  "queryFn" | "queryKey"
> & { name: Invoke; args?: Invokes[Invoke]["args"]; queryKey?: QueryKey }) {
  const query = useQuery({
    queryFn: () => backend(name, args),
    queryKey: name.split("_"),
    ...params,
  });

  return query;
}

type TVarsType<
  Args extends Invokes[keyof Invokes]["args"],
  TArgs extends Partial<Record<string, unknown>>,
> = keyof Omit<Args, keyof TArgs> extends never
  ? void // بدون پارامتر
  : Omit<Args, keyof TArgs>; // همان تعریف قبلی

export function useBackendMutation<
  Invoke extends keyof Invokes,
  // biome-ignore lint/complexity/noBannedTypes: with {} evey magical type works fine
  TArgs extends Partial<Invokes[Invoke]["args"]> = {},
>({
  name,
  args,
  onError,
  ...params
}: Omit<
  UseMutationOptions<
    Invokes[Invoke]["returns"], // TData
    InvokeError<Invokes[Invoke]["custom_error"]>, // TError
    TVarsType<Invokes[Invoke]["args"], TArgs> // TVariables
  >,
  "mutationFn"
> & { name: Invoke; args?: TArgs }) {
  type TVars = TVarsType<Invokes[Invoke]["args"], TArgs>;

  const mutation = useMutation<
    Invokes[Invoke]["returns"], // TData
    InvokeError<Invokes[Invoke]["custom_error"]>, // TError
    TVars // TVariables
  >({
    mutationFn: (variables: TVars) =>
      backend(name, { ...args, ...variables } as Invokes[Invoke]["args"]),
    mutationKey: name.split("_"),
    onError: (error, vars, res, context) => {
      if (onError) {
        onError(error, vars, res, context);
      } else {
        const displayError = errorText(error.code);
        toast.error(displayError.title, {
          description: displayError.description,
        });
        sendNotification({
          body: displayError.description,
          title: displayError.title,
        });
        if (typeof error.data == "string"){

          invoke("error", { message: error.data });
        }
      }
    },
    ...params,
  });

  return mutation;
}
