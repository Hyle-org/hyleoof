type HttpMethod = "GET" | "POST" | "PUT" | "DELETE";

interface ApiRequestConfig {
  baseUrl: string;
  endpoint: string;
  method?: HttpMethod;
  body?: Record<string, any>;
}



export function createApiRequest<T>({
  baseUrl,
  endpoint,
  method = "GET",
  body,
}: ApiRequestConfig) {
  return async (): Promise<T> => {
    const response = await fetch(`${baseUrl}${endpoint}`, {
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
