import { FormEvent } from "react";
import { useMutation, UseMutationOptions } from "@tanstack/react-query";

type FormDataHandler<TParams, TResponse> = (
  params: TParams,
) => Promise<TResponse>;

/**
 * A custom hook that handles form submissions with React Query mutations.
 * It automatically converts FormData to the expected parameter type and provides
 * a submit handler that prevents default form behavior.
 *
 * @param apiCall A function that takes form data as parameters and makes an API request.
 *                It should convert the FormData entries into the expected parameter type TParams
 *                and return a Promise resolving to TResponse.
 * @param options Optional react-query mutation options excluding the mutationFn
 * @returns An object containing:
 *          - mutation: The React Query mutation object
 *          - handleSubmit: A form submission handler that prevents default behavior
 */
export function useFormSubmission<TParams, TResponse>(
  apiCall: FormDataHandler<TParams, TResponse>,
  options?: Omit<UseMutationOptions<TResponse, Error, FormData>, "mutationFn">,
) {
  const mutation = useMutation({
    ...options,
    mutationFn: (formData: FormData) => {
      const formDataObject = Object.fromEntries(formData) as TParams;
      return apiCall(formDataObject);
    },
  });

  const handleSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    mutation.mutate(new FormData(event.currentTarget));
  };

  return {
    mutation,
    handleSubmit,
  };
}
