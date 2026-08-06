import errorsJson from '@/messages/errors.json';

const errors = new Map(Object.entries(errorsJson));
export const errorText = (code: string) =>
    errors.get(code) ?? errorsJson.ERROR_NOT_IMPLEMENTED;
