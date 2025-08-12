import {XMLParser} from "fast-xml-parser";


const parser = new XMLParser({
    ignoreAttributes: false,
    attributeNamePrefix: "@_",
    isArray: (name) => name === "string" // force <string> to always be an array
});
let translations = {};
let currentLang = "en";

export async function loadLanguage(lang) {
    const res = await fetch(`/${lang}.xml`);
    if (!res.ok) {
        throw new Error(`Could not load ${lang}.xml`);
    }
    const xmlText = await res.text();
    const json = parser.parse(xmlText);

    const entries = json.resources.string;
    const map = {};

    if (Array.isArray(entries)) {
        entries.forEach(entry => {
            map[entry["@_id"]] = entry["#text"]
        });
    } else {
        map[entries["@_id"]] = entries["#text"];
    }


    translations = map;
    currentLang = lang;
}

export function t(key) {
    return translations[key] || key;
}

export function getCurrentLang() {
    return currentLang;
}