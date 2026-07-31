import * as React from 'react';
import ConfigContext from './ConfigContext';
import { Config } from '../../modelos/config';

export interface ConfigProviderProps {
  children?: React.ReactNode;
}

export default function ConfigProvider(props: ConfigProviderProps) {
  const { children } = props;
  const [config, setConfigState] = React.useState<Config | null>(null);

  const setConfig = React.useCallback((cfg: Config | null) => {
    setConfigState(cfg);
  }, []);

  const getConfig = React.useCallback(() => {
    if (config === null) {
      throw new Error(
        'No se puede realizar ninguna operación sin una configuración');
    }

    return config;
  }, [config]);

  const contextValue = React.useMemo(() => ({
    setConfig: setConfig,
    getConfig: getConfig,
  }), [setConfig, getConfig]);

  return (
    <ConfigContext.Provider value={contextValue}>
      {children}
    </ConfigContext.Provider>
  );
}
