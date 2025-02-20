import { createContext, useContext, useState, useCallback, ReactNode } from 'react';
import { v4 as uuidv4 } from 'uuid';

interface Notification {
  id: string;
  message: string;
}

interface NotificationContextType {
  notifications: Notification[];
  addNotification: (message: string) => void;
  removeNotification: (id: string) => void;
}

const NotificationContext = createContext<NotificationContextType | undefined>(undefined);

export const NotificationProvider = ({ children }: { children: ReactNode }) => {
  const [notifications, setNotifications] = useState<Notification[]>([]);

  const addNotification = useCallback((message: string) => {
    setNotifications((prevNotifications) => [
      ...prevNotifications,
      { id: uuidv4(), message },
    ]);

    /*    setTimeout(() => {
          setNotifications((prevNotifications) =>
            prevNotifications.filter((notification) => notification.message !== message)
          );
        }, 6000);*/
  }, []);

  const removeNotification = useCallback((id: string) => {
    setNotifications((prevNotifications) =>
      prevNotifications.filter((notification) => notification.id !== id)
    );
  }, []);

  return (
    <NotificationContext.Provider value={{ notifications, addNotification, removeNotification }}>
      {children}
    </NotificationContext.Provider>
  );
};

export const useNotification = () => {
  const context = useContext(NotificationContext);
  if (!context) {
    throw new Error('useNotification must be used within a NotificationProvider');
  }
  return context;
};

