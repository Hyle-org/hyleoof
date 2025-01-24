import { SelectHTMLAttributes } from "react";
import styles from "./Select.module.css";

interface SelectProps extends SelectHTMLAttributes<HTMLSelectElement> {
  labelText: string;
  hintText: string;
}

export default function Select({
  children,
  labelText,
  hintText,
  onChange,
  ...props
}: SelectProps) {
  return (
    <div className={styles.tokenSelector}>
      <label>{labelText}</label>
      <select className={styles.tokenDropdown} onChange={onChange} {...props}>
        {children}
      </select>
      <p>{hintText}</p>
    </div>
  );
}
