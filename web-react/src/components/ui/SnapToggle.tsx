import React from 'react';
import './SnapToggle.css';
import FlaskFox from '@/assets/flask_fox.svg?react';
import MetamaskFox from '@/assets/metamask_fox.svg?react';

interface ToggleButtonProps {
  isToggled: boolean;
  onToggle: () => void;
}

const SnapToggle: React.FC<ToggleButtonProps> = ({ isToggled, onToggle }) => {
  return (
    <div className="toggle-button" onClick={onToggle}>
      {isToggled ? <FlaskFox className="logo" /> : <MetamaskFox className="logo" />}
    </div>
  );
};

export default SnapToggle;

