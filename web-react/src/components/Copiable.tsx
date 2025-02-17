import { shortenString } from '@/utils/shortenString';
import React, { useState } from 'react';

const Copiable = ({ text, size }: { text: string, size: number | null }) => {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (error) {
      console.error("Erreur lors de la copie : ", error);
    }
  };

  return (
    <div style={{ display: 'flex', alignItems: 'center' }}>
      <span onClick={handleCopy} style={{ cursor: 'pointer' }}>{shortenString(text, size || -1)}</span>
      <button
        onClick={handleCopy}
        title={copied ? "Copied!" : "Copy"}
        style={{
          marginLeft: '8px',
          background: 'none',
          border: 'none',
          cursor: 'pointer',
          padding: 0,
        }}
      >
        <span style={{ color: 'white' }}>
          ⎘
        </span>
      </button>
    </div>
  );
};

export default Copiable;

