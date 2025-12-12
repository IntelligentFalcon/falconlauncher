import { InvokeArgs } from '@tauri-apps/api/core';

type InvokeError<T = unknown> = {
  code: number;
  message: string;
  data?: T;
};

type WithDefaultError<T> = T &
  Record<
    keyof T,
    {
      custom_error: T extends { custom_error: infer E } ? E : {};
    }
  >;

type Invokes = WithDefaultError<{
  toggle_mod: {
    args: {
      modInfo: ModInfo;
      toggle: boolean;
    };
    returns: void;
  };
  delete_mod: {
    args: ModInfo;
    returns: void;
  };
  get_mods: {
    args: undefined;
    returns: ModInfo[];
  };
}>;
