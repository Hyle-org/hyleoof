type HttpMethod = "GET" | "POST" | "PUT" | "DELETE";

interface ApiRequestConfig {
  baseUrl: string;
  endpoint: string;
  method?: HttpMethod;
  body?: Record<string, any>;
}

interface ApiError {
  message: string;
  status: number;
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
      const error: ApiError = {
        message: await response.text(),
        status: response.status,
      };

      throw error;
    }

    return response.json();
  };
}
