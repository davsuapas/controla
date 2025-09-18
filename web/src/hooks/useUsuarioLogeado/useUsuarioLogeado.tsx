// useUser.js
import * as React from 'react';
import UsuarioContext from './UsuarioLogeadoContext';
import { Usuario } from '../../modelos/usuarios';

export interface SetUsrLogeado {
  /**
   * Asignar un usuario al contexto.
   *
   * @param user Los datos del usuario a asignar.
   */
  (user: Usuario | null): void;
}

export interface GetUsrLogeado {
  /**
   * Obtener el usuario actual del contexto.
   *
   * @returns Los datos del usuario actual o null si no hay usuario.
   */
  (): Usuario;
}

export interface HayUsrLogeado {
  /**
   * Verificar si hay un usuario asignado en el contexto.
   *
   * @returns true si hay usuario asignado, false en caso contrario.
   */
  (): boolean;
}

export interface UseUsuarioLogeado {
  setUsrLogeado: SetUsrLogeado;
  getUsrLogeado: GetUsrLogeado;
  hayUsrLogeado: HayUsrLogeado;
}

export default function useUsuarioLogeado(): UseUsuarioLogeado {
  const userContext = React.useContext(UsuarioContext);
  if (!userContext) {
    throw new Error('User context was used without a provider.');
  }
  return userContext;
}