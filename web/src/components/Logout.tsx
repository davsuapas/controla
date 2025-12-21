import { api } from "../api/fabrica";
import { useEffect } from "react";
import useUsuarioLogeado from "../hooks/useUsuarioLogeado/useUsuarioLogeado";
import { NetErrorControlado } from "../net/interceptor";
import React from "react";
import Box from "@mui/material/Box";
import CircularProgress from "@mui/material/CircularProgress";
import Alert from "@mui/material/Alert";
import PageContainer from "./PageContainer";
import { logError } from "../error";
import { useIsMounted } from '../hooks/useComponentMounted';
import { URL_BASE_ROUTER } from "../config";

export default function Logout() {
  const { setUsrLogeado, getUsrLogeado } = useUsuarioLogeado()
  const isMounted = useIsMounted();

  const [isLoading, setIsLoading] = React.useState(true);
  const [error, setError] = React.useState<Error | null>(null);

  const realizarLogout = React.useCallback(async () => {
    setError(null);
    setIsLoading(true);

    try {
      let usr = getUsrLogeado();

      await api().usuarios.logout(usr.id.toString());
      setUsrLogeado(null);

      if (isMounted.current) {
        setIsLoading(false);
      };

      // Forzamos a eliminar caches. Liberamos memoria
      window.location.replace(URL_BASE_ROUTER);
    } catch (error) {
      if (!(error instanceof NetErrorControlado)) {
        logError('logout', error);

        if (isMounted.current) {
          setError(Error('Error inesperado al cerrar la sesión'));
        };
      }
    }

    if (isMounted.current) {
      setIsLoading(false);
    };

  }, [setUsrLogeado, getUsrLogeado]);

  useEffect(() => {
    realizarLogout();
  }, [realizarLogout]);

  const render = React.useMemo(() => {
    if (isLoading) {
      return (
        <Box
          sx={{
            flex: 1,
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            justifyContent: 'center',
            width: '100%',
            m: 1,
          }}
        >
          <CircularProgress />
        </Box>
      );
    }
    if (error) {
      return (
        <Box sx={{ flexGrow: 1 }}>
          <Alert severity="error">{error.message}</Alert>
        </Box>
      );
    }

    return null;
  }, [isLoading, error]);

  return (
    <PageContainer>
      <Box sx={{ display: 'flex', flex: 1 }}>{render}</Box>
    </PageContainer>
  );
}