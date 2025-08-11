import {X} from 'lucide-react';
import {invoke} from "@tauri-apps/api/core";
import {useState} from "react";

const MicrosoftLogo = () => (<svg width="21" height="21" viewBox="0 0 21 21" className="mr-2" aria-hidden="true">
    <path fill="#f25022" d="M1 1h9v9H1z"/>
    <path fill="#00a4ef" d="M1 11h9v9H1z"/>
    <path fill="#7fba00" d="M11 1h9v9h-9z"/>
    <path fill="#ffb900" d="M11 11h9v9h-9z"/>
</svg>);

export default function LoginPopup({isOpen, onClose}) {
    if (!isOpen) {
        return null;
    }
    const [username, setUsername] = useState("")
    return (<div className="fixed inset-0 bg-black bg-opacity-70 flex justify-center items-center z-50">

        <div className="bg-gray-800 p-8 rounded-lg shadow-xl w-full max-w-sm relative text-gray-200">

            <button onClick={onClose}
                    className="absolute top-4 right-4 text-gray-400 hover:text-white transition-colors">
                <X size={24}/>
            </button>

            <h2 className="text-3xl font-bold text-center mb-6">Login</h2>

            <form>
                <div className="mb-4">
                    <input
                        type="text"
                        placeholder="Username"
                        className="w-full p-3 bg-gray-900 border border-gray-700 rounded text-gray-200 focus:outline-none focus:ring-2 focus:ring-indigo-500"
                        onInput={event => {
                            setUsername(event.target.value);
                            invoke("set_username", {username: event.target.value}).catch("Guess what? i couldn't save your username");
                        }}
                    />
                </div>

                <button
                    type="submit"
                    className="w-full mb-4 p-3 bg-green-600 hover:bg-green-700 text-white font-bold rounded transition-colors"
                    onClick={event => {
                        invoke("create_offline_profile", {username: username}).catch("Oh failed to create profile!");
                        onClose
                    }}
                >
                    Log in
                </button>

                <button
                    type="button"
                    className="w-full p-3 bg-white text-gray-800 font-semibold rounded flex items-center justify-center hover:bg-gray-200 transition-colors"
                >
                    <MicrosoftLogo/>
                    Sign in with Microsoft
                </button>
            </form>
        </div>
    </div>);
}