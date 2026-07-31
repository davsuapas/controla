import * as React from 'react';
import ConfigContext from './ConfigContext';
import { Config } from '../../modelos/config';

export interface SetConfig {
  (config: Config | null): void;
}

export interface GetConfig {
  (): Config;
}

export interface UseConfig {
  setConfig: SetConfig;
  getConfig: GetConfig;
}

export default function useConfig(): UseConfig {
  const configContext = React.useContext(ConfigContext);
  if (!configContext) {
    throw new Error('Config context was used without a provider.');
  }
  return configContext;
}
