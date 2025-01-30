type Camelize<T extends string> = T extends `${infer A}_${infer B}` ? `${A}${Camelize<Capitalize<B>>}` : T

export type CamelizeKeys<T extends object> = {
  [key in keyof T as key extends string ? Camelize<key> : key]: T[key] extends object ? CamelizeKeys<T[key]> : T[key]
}

// Utility function to convert snake_case to camelCase
function toCamelCase(str: string): string {
  return str.replace(/_([a-z])/g, (_, letter) => letter.toUpperCase());
}

// Function to recursively transform the keys of an object
export function camelizeKeys<T extends object>(obj: T): CamelizeKeys<T> {
  if (Array.isArray(obj)) {
    return obj.map(item => camelizeKeys(item)) as any;
  } else if (obj !== null && obj.constructor === Object) {
    return Object.keys(obj).reduce((result, key) => {
      const camelKey = toCamelCase(key);
      result[camelKey] = camelizeKeys((obj as any)[key]);
      return result;
    }, {} as any);
  }
  return obj as any;
}