import React from 'react';
import Notification from './Notification';
import { useNotification } from '@/hooks/NotificationContext'
import './NotificationList.css';

const NotificationList: React.FC = () => {
  const { notifications, removeNotification } = useNotification();

  console.log("notifications", notifications);

  return (
    <div className="notification-list">
      {notifications.map((notification) => (
        <Notification
          key={notification.id}
          notification={notification}
          onClose={removeNotification}
        />
      ))}
    </div>
  );
};

export default NotificationList;

