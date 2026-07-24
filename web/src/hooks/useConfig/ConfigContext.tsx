import * as React from 'react';
import { SetConfig, GetConfig } from './useConfig';

const ConfigContext = React.createContext<{
  setConfig: SetConfig;
  getConfig: GetConfig;
} | null>(null);

export default ConfigContext;
