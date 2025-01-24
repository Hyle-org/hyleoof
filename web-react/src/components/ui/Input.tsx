import { InputHTMLAttributes } from "react";
import styles from "./Input.module.css";

interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  labelText: string;
  suffixText?: string;
}

console.log("Styles object:", styles);

export default function Input({
  labelText,
  suffixText,
  type,
  placeholder,
  ...props
}: InputProps) {
  return (
    <div>
      <label>{labelText}</label>
      <div className={styles.inputWrapper}>
        <input
          type={type}
          className={styles.inputText}
          placeholder={placeholder}
          {...props}
        />
        <span className={styles.inputSuffix}>{suffixText}</span>
      </div>
    </div>
  );
}
