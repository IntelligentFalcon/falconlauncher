import { invoke } from '@tauri-apps/api/core';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

// @ts-ignore
const MicrosoftLogo = () => (
  <svg
    width="21"
    height="21"
    viewBox="0 0 21 21"
    className="mr-2"
    aria-hidden="true"
  >
    <path fill="#f25022" d="M1 1h9v9H1z" />
    <path fill="#00a4ef" d="M1 11h9v9H1z" />
    <path fill="#7fba00" d="M11 1h9v9h-9z" />
    <path fill="#ffb900" d="M11 11h9v9h-9z" />
  </svg>
);

export function LoginPopup({ close }: { close: () => void }) {
  const { t } = useTranslation();
  const [username, setUsername] = useState('');

  return (
    <form>
      <div className="mb-4">
        <input
          type="text"
          placeholder="Username"
          className="w-full p-3 bg-gray-900 border border-gray-700 rounded-sm text-gray-200 focus:outline-hidden focus:ring-2 focus:ring-indigo-500"
          onChange={(event) => {
            setUsername(event.target.value);
            invoke('set_username', { username: event.target.value }).catch(
              () => "Guess what? i couldn't save your username"
            );
          }}
        />
      </div>

      <button
        type="submit"
        className="w-full mb-4 p-3 bg-green-600 hover:bg-green-700 text-white font-bold rounded-sm transition-colors"
        onClick={() => {
          invoke('create_offline_profile', { username: username }).catch(
            () => 'Oh failed to create profile!'
          );
          close();
        }}
      >
        {t('login')}
      </button>

      {/*<button*/}
      {/*    type="button"*/}
      {/*    className="w-full p-3 bg-white text-gray-800 font-semibold rounded-sm flex items-center justify-center hover:bg-gray-200 transition-colors"*/}
      {/*>*/}
      {/*    <MicrosoftLogo/>*/}
      {/*    Sign in with Microsoft*/}
      {/*</button>*/}
    </form>
  );
}
