import * as React from 'react';
import Alert from '@mui/material/Alert';
import Box from '@mui/material/Box';
import CircularProgress from '@mui/material/CircularProgress';
import Grid from '@mui/material/Grid';
import Paper from '@mui/material/Paper';
import Typography from '@mui/material/Typography';
import Chip from '@mui/material/Chip';
import PageContainer from './PageContainer';
import { Usuario } from '../modelos/usuarios';
import { api } from '../api/fabrica';
import useUsuarioLogeado from "../hooks/useUsuarioLogeado/useUsuarioLogeado";
import { NetErrorControlado } from '../net/interceptor';
import { FULL_HEIGHT_WIDTH } from '../context/DashboardSidebarContext';
import { logError } from '../error';
import { useIsMounted } from '../hooks/useComponentMounted';

export default function UsuarioShow() {
  const { getUsrLogeado } = useUsuarioLogeado();
  const isMounted = useIsMounted();

  const [usuario, setUsuario] = React.useState<Usuario | null>(null);
  const [isLoading, setIsLoading] = React.useState(true);
  const [error, setError] = React.useState<Error | null>(null);

  const loadData = React.useCallback(async () => {
    setError(null);
    setIsLoading(true);

    try {
      const showData = await api().usuarios.usuario(
        getUsrLogeado().id.toString());

      if (isMounted.current) {
        setUsuario(showData);
      }
    } catch (error) {
      if (!(error instanceof NetErrorControlado)) {
        logError('usuario-visualizar.cargar', error);
        if (isMounted.current) {
          setError(Error('Error inesperado al visualizar el usuario'));
        }
      }
    }

    if (isMounted.current) {
      setIsLoading(false);
    }
  }, [getUsrLogeado]);

  React.useEffect(() => {
    loadData();
  }, []);

  const renderShow = React.useMemo(() => {
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

    return usuario ? (
      <Box sx={{ flexGrow: 1, ...FULL_HEIGHT_WIDTH }}>
        <Grid container spacing={2} sx={{ width: '100%' }}>
          <Grid size={{ xs: 12, sm: 6 }}>
            <Paper sx={{ px: 2, py: 1 }}>
              <Typography variant="h6" sx={{ fontSize: { xs: '0.875rem', sm: '1rem' } }}>
                ID
              </Typography>
              <Typography
                variant="h2"
                sx={{
                  mb: 1,
                  fontSize: { xs: '1.25rem', sm: '2rem' },
                  wordBreak: 'break-word',
                  overflowWrap: 'break-word'
                }}
              >
                {usuario.id}
              </Typography>
            </Paper>
          </Grid>
          <Grid size={{ xs: 12, sm: 6 }}>
            <Paper sx={{ px: 2, py: 1 }}>
              <Typography variant="h6" sx={{ fontSize: { xs: '0.875rem', sm: '1rem' } }}>
                DNI
              </Typography>
              <Typography
                variant="h2"
                sx={{
                  mb: 1,
                  fontSize: { xs: '1.25rem', sm: '2rem' },
                  wordBreak: 'break-word',
                  overflowWrap: 'break-word'
                }}
              >
                {usuario.dni}
              </Typography>
            </Paper>
          </Grid>
          <Grid size={{ xs: 12, sm: 6 }}>
            <Paper sx={{ px: 2, py: 1 }}>
              <Typography variant="h6" sx={{ fontSize: { xs: '0.875rem', sm: '1rem' } }}>
                Nombre
              </Typography>
              <Typography
                variant="h2"
                sx={{
                  mb: 1,
                  fontSize: { xs: '1.25rem', sm: '2rem' },
                  wordBreak: 'break-word',
                  overflowWrap: 'break-word'
                }}
              >
                {usuario.nombre}
              </Typography>
            </Paper>
          </Grid>
          <Grid size={{ xs: 12, sm: 6 }}>
            <Paper sx={{ px: 2, py: 1 }}>
              <Typography variant="h6" sx={{ fontSize: { xs: '0.875rem', sm: '1rem' } }}>
                Primer apellido
              </Typography>
              <Typography
                variant="h2"
                sx={{
                  mb: 1,
                  fontSize: { xs: '1.25rem', sm: '2rem' },
                  wordBreak: 'break-word',
                  overflowWrap: 'break-word'
                }}
              >
                {usuario.primerApellido}
              </Typography>
            </Paper>
          </Grid>
          <Grid size={{ xs: 12, sm: 6 }}>
            <Paper sx={{ px: 2, py: 1 }}>
              <Typography variant="h6" sx={{ fontSize: { xs: '0.875rem', sm: '1rem' } }}>
                Segundo apellido
              </Typography>
              <Typography
                variant="h2"
                sx={{
                  mb: 1,
                  fontSize: { xs: '1.25rem', sm: '2rem' },
                  wordBreak: 'break-word',
                  overflowWrap: 'break-word'
                }}
              >
                {usuario.segundoApellido}
              </Typography>
            </Paper>
          </Grid>
          <Grid size={{ xs: 12, sm: 6 }}>
            <Paper sx={{ px: 2, py: 1 }}>
              <Typography sx={{ fontSize: { xs: '0.875rem', sm: '1rem' } }}>
                Email
              </Typography>
              <Typography
                sx={{
                  mb: 1,
                  fontSize: { xs: '1.25rem', sm: '2rem' },
                  wordBreak: 'break-word',
                  overflowWrap: 'break-word'
                }}
              >
                {usuario.email}
              </Typography>
            </Paper>
          </Grid>
          <Grid size={{ xs: 12, sm: 6 }}>
            <Paper sx={{ px: 2, py: 1 }}>
              <Typography variant="h6" sx={{ fontSize: { xs: '0.875rem', sm: '1rem' } }}>
                Roles
              </Typography>
              <Typography
                variant="h2"
                sx={{
                  mb: 1,
                  fontSize: { xs: '1.25rem', sm: '2rem' },
                  wordBreak: 'break-word',
                  overflowWrap: 'break-word'
                }}
              >
                {usuario.roles.map((rol) => (
                  <Chip
                    key={rol.id}
                    label={rol.nombre}
                    size="medium"
                    variant="outlined"
                  />
                ))}
              </Typography>
            </Paper>
          </Grid>
          <Grid size={{ xs: 12, sm: 6 }}>
            <Paper sx={{ px: 2, py: 1 }}>
              <Typography variant="h6" sx={{ fontSize: { xs: '0.875rem', sm: '1rem' } }}>
                Primer acceso
              </Typography>
              <Typography
                variant="h2"
                sx={{
                  mb: 1,
                  fontSize: { xs: '1.25rem', sm: '2rem' },
                  wordBreak: 'break-word',
                  overflowWrap: 'break-word'
                }}
              >
                {usuario.inicioToStr()}
              </Typography>
            </Paper>
          </Grid>
        </Grid>
      </Box>
    ) : null;
  }, [
    isLoading,
    error,
    usuario,
  ]);

  return (
    <PageContainer
      title={'Perfil de usuario'}
    >
      <Box sx={{ display: 'flex', flex: 1, width: '100%' }}>{renderShow}</Box>
    </PageContainer>
  );
}