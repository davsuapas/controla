// UserContext.js
import * as React from 'react';
import { SetUsrLogeado, GetUsrLogeado, HayUsrLogeado } from './useUsuarioLogeado';

const UserContext = React.createContext<{
  setUsrLogeado: SetUsrLogeado;
  getUsrLogeado: GetUsrLogeado;
  hayUsrLogeado: HayUsrLogeado;
} | null>(null);

export default UserContext;