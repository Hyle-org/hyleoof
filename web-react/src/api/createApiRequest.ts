type HttpMethod = "GET" | "POST" | "PUT" | "DELETE";

interface ApiRequestConfig {
  endpoint: string;
  method?: HttpMethod;
  body?: Record<string, any>;
}

const BASE_URL = import.meta.env.VITE_SERVER_URL;

export function createApiRequest<T>({
  endpoint,
  method = "GET",
  body,
}: ApiRequestConfig) {
  return async (): Promise<T> => {
    const response = await fetch(`${BASE_URL}${endpoint}`, {
      method,
      headers: {
        "Content-Type": "application/json",
      },
      ...(body && { body: JSON.stringify(body) }),
    });

    if (!response.ok) {
      throw new Error("Network response was not ok");
    }

    return response.json();
  };
}
