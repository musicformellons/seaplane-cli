import { decode } from '../utils/base64';

export type KeyValue = {
  key: string;
  value: string;
};

export type Key = {
  key: string;
};

export type KeyValuePage = {
  keyValuePairs: KeyValue[];
  nextKey?: Key | null;
};

export const mapKey = (key?: string) => {
  if (!key) {
    return null;
  }

  return { key: decode(key) };
};

export const mapToKeyValue = (json: any) => ({
  // eslint-disable-line
  key: decode(json.key),
  value: decode(json.value),
});

export const mapToKeyValuePage = (json: any) => ({
  // eslint-disable-line
  keyValuePairs: json.kvs.map((kv: any) => mapToKeyValue(kv)), // eslint-disable-line
  nextKey: mapKey(json.next_key),
});
