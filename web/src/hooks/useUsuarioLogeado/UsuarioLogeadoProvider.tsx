import * as React from 'react';
import UserContext from './UsuarioLogeadoContext';
import { Usuario } from '../../modelos/usuarios';

export interface UserProviderProps {
  children?: React.ReactNode;
}

/**
 * Provider for User context.
 * The subtree of this component can use the `useUser` hook to
 * access the user management API.
 */
export default function UsuarioLogeadoProvider(props: UserProviderProps) {
  const { children } = props;
  const [user, setUserState] = React.useState<Usuario | null>(null);

  const setUsrLogeado = React.useCallback((usuario: Usuario | null) => {
    setUserState(usuario);
  }, []);

  const getUsrLogeado = React.useCallback(() => {
    if (user === null) {
      throw new Error(
        'No se puede realizar ninguna operación sin un usuario logeado');
    }

    return user;
  }, [user]);

  const hayUsrLogeado = React.useCallback(() => {
    return user !== null;
  }, [user]);

  const contextValue = React.useMemo(() => ({
    setUsrLogeado: setUsrLogeado,
    getUsrLogeado: getUsrLogeado,
    hayUsrLogeado: hayUsrLogeado
  }), [setUsrLogeado, getUsrLogeado, hayUsrLogeado]);

  return (
    <UserContext.Provider value={contextValue}>
      {children}
    </UserContext.Provider>
  );
}