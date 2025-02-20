import React, { useEffect } from 'react';
import './Notification.css';

interface NotificationProps {
  notification: { id: string; message: string };
  onClose: (id: string) => void;
}

const Notification: React.FC<NotificationProps> = ({ notification, onClose }) => {
  useEffect(() => {
    const timer = setTimeout(() => {
      onClose(notification.id);
    }, 6000);

    return () => clearTimeout(timer);
  }, [notification.id, onClose]);

  const handleClick = () => {
    onClose(notification.id);
  };

  return (
    <div className="notification show" onClick={handleClick}>
      {notification.message}
    </div>
  );
};

export default Notification;

