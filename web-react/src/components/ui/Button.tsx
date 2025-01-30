import { ButtonHTMLAttributes } from "react";
import styles from "./Button.module.css";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {}

export default function Button({ type, children }: ButtonProps) {
  return (
    <button className={styles.submitButton} type={type}>
      {children}
    </button>
  );
}
