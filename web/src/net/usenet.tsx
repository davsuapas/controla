import { useContext } from 'react';
import React, { ReactNode } from 'react';
import { useNavigate } from 'react-router';
import { axios, configurarUI } from './interceptor';
import { useDialogs } from '../hooks/useDialogs/useDialogs';

export const useNet = () => {
  const context = useContext(NetContext);
  if (!context) {
    throw new Error('useNet debe usarse dentro de NetProvider');
  }
  return context;
};

interface NetContextType {
  axios: typeof axios;
}

const NetContext = React.createContext<NetContextType | undefined>(undefined);

interface NetProviderProps {
  children: ReactNode;
}

export const NetProvider: React.FC<NetProviderProps> = ({ children }) => {
  const dialogs = useDialogs();
  const navigate = useNavigate();

  React.useEffect(() => {
    configurarUI(dialogs, () => navigate('/login'));
  }, [dialogs, navigate]);

  const contextValue = { axios };

  return (
    <NetContext.Provider value={contextValue} >
      {children}
    </NetContext.Provider>
  );
};