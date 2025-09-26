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

export default function UsuarioShow() {
  const { getUsrLogeado } = useUsuarioLogeado();

  const [usuario, setUsuario] = React.useState<Usuario | null>(null);
  const [isLoading, setIsLoading] = React.useState(true);
  const [error, setError] = React.useState<Error | null>(null);

  const loadData = React.useCallback(async () => {
    setError(null);
    setIsLoading(true);

    try {
      const showData = await api().usuarios.usuario(
        getUsrLogeado().id.toString());
      setUsuario(showData);
    } catch (error) {
      if (!(error instanceof NetErrorControlado)) {
        logError('usuariovisualizar.cargar', error);
        setError(Error('Error inesperado al visualizar el usuario'));
      }
    }

    setIsLoading(false);
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
              <Typography variant="h5">ID</Typography>
              <Typography variant="body1" sx={{ mb: 1 }}>
                {usuario.id}
              </Typography>
            </Paper>
          </Grid>
          <Grid size={{ xs: 12, sm: 6 }}>
            <Paper sx={{ px: 2, py: 1 }}>
              <Typography variant="h5">DNI</Typography>
              <Typography variant="body1" sx={{ mb: 1 }}>
                {usuario.dni}
              </Typography>
            </Paper>
          </Grid>
          <Grid size={{ xs: 12, sm: 6 }}>
            <Paper sx={{ px: 2, py: 1 }}>
              <Typography variant="h5">NOMBRE</Typography>
              <Typography variant="body1" sx={{ mb: 1 }}>
                {usuario.nombre}
              </Typography>
            </Paper>
          </Grid>
          <Grid size={{ xs: 12, sm: 6 }}>
            <Paper sx={{ px: 2, py: 1 }}>
              <Typography variant="h5">PRIMER APELLIDO</Typography>
              <Typography variant="body1" sx={{ mb: 1 }}>
                {usuario.primerApellido}
              </Typography>
            </Paper>
          </Grid>
          <Grid size={{ xs: 12, sm: 6 }}>
            <Paper sx={{ px: 2, py: 1 }}>
              <Typography variant="h5">SEGUNDO APELLIDO</Typography>
              <Typography variant="body1" sx={{ mb: 1 }}>
                {usuario.segundoApellido}
              </Typography>
            </Paper>
          </Grid>
          <Grid size={{ xs: 12, sm: 6 }}>
            <Paper sx={{ px: 2, py: 1 }}>
              <Typography variant="h5">EMAIL</Typography>
              <Typography variant="body1" sx={{ mb: 1 }}>
                {usuario.email}
              </Typography>
            </Paper>
          </Grid>
          <Grid size={{ xs: 12, sm: 6 }}>
            <Paper sx={{ px: 2, py: 1 }}>
              <Typography variant="h5">ROLES</Typography>
              <Typography variant="body1" sx={{ mb: 1 }}>
                {usuario.roles.map((rol) => (
                  <Chip
                    key={rol.id}
                    label={rol.nombre}
                    size="small"
                    variant="outlined"
                  />
                ))}
              </Typography>
            </Paper>
          </Grid>
          <Grid size={{ xs: 12, sm: 6 }}>
            <Paper sx={{ px: 2, py: 1 }}>
              <Typography variant="h5">PRIMER ACCESO</Typography>
              <Typography variant="body1" sx={{ mb: 1 }}>
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