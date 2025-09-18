import * as React from 'react';
import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import FormLabel from '@mui/material/FormLabel';
import FormControl from '@mui/material/FormControl';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';
import Stack from '@mui/material/Stack';
import MuiCard from '@mui/material/Card';
import { styled } from '@mui/material/styles';
import ColorModeSelect from '../theme/ColorModeSelect';
import SitemarkIcon from './SitemarkIcon';
import { api } from '../api/usuarios';
import { useDialogs } from '../hooks/useDialogs/useDialogs';
import { useLocation, useNavigate } from 'react-router';
import { RolID } from '../modelos/usuarios';
import useUsuarioLogeado from '../hooks/useUsuarioLogeado/useUsuarioLogeado';
import { AxiosError } from 'axios';
import { useRutasDashboard } from '../controla';

const Card = styled(MuiCard)(({ theme }) => ({
  display: 'flex',
  flexDirection: 'column',
  alignSelf: 'center',
  width: '100%',
  padding: theme.spacing(4),
  gap: theme.spacing(2),
  margin: 'auto',
  [theme.breakpoints.up('sm')]: {
    maxWidth: '450px',
  },
  boxShadow:
    'hsla(220, 30%, 5%, 0.05) 0px 5px 15px 0px, hsla(220, 25%, 10%, 0.05) 0px 15px 35px -5px',
  ...theme.applyStyles('dark', {
    boxShadow:
      'hsla(220, 30%, 5%, 0.5) 0px 5px 15px 0px, hsla(220, 25%, 10%, 0.08) 0px 15px 35px -5px',
  }),
}));

const SignInContainer = styled(Stack)(({ theme }) => ({
  height: 'calc((1 - var(--template-frame-height, 0)) * 100dvh)',
  minHeight: '100%',
  padding: theme.spacing(2),
  [theme.breakpoints.up('sm')]: {
    padding: theme.spacing(4),
  },
  '&::before': {
    content: '""',
    display: 'block',
    position: 'absolute',
    zIndex: -1,
    inset: 0,
    backgroundImage:
      'radial-gradient(ellipse at 50% 50%, hsl(210, 100%, 97%), hsl(0, 0%, 100%))',
    backgroundRepeat: 'no-repeat',
    ...theme.applyStyles('dark', {
      backgroundImage:
        'radial-gradient(at 50% 50%, hsla(210, 100%, 16%, 0.5), hsl(220, 30%, 5%))',
    }),
  },
}));

const navegacionPorRol = new Map<RolID, string>([
  [RolID.Admin, '/usuarios'],
  [RolID.Empleado, '/usuarios'],
  [RolID.Gestor, '/usuarios'],
  [RolID.Director, '/usuarios'],
  [RolID.Registrador, '/usuarios'],
  [RolID.Inspector, '/usuarios'],
  [RolID.Configurador, '/usuarios'],
]);

// Logea el usuario y lo redirige a la pantalla correspondiente
// de acuerdo a su rol
export default function Login() {
  const { setUsrLogeado } = useUsuarioLogeado()
  const dialogo = useDialogs();
  const navegar = useNavigate();
  const rutasDashboard = useRutasDashboard();
  const location = useLocation();

  const handleSubmit = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();

    const data = new FormData(event.currentTarget);
    const dni = data.get('dni') as string;

    try {
      const usr = await api().usuarios.login(
        dni,
        data.get('password') as string);

      // Obtener la ruta de destino desde el estado de navegación
      // Extraer el primer segmento de la ruta
      const paginaOrigen = location.state?.redirect || '/';
      const primerSegmento = paginaOrigen.split('/')[1];

      // Verificar si viene de una ruta del Dashboard
      if (primerSegmento === '' || !rutasDashboard.includes(primerSegmento)) {
        for (const [rol_id, ruta] of navegacionPorRol.entries()) {
          if (usr.hayRol(rol_id)) {
            setUsrLogeado(usr);
            navegar(ruta, { replace: true });
            return;
          }
        }
      } else {
        // Redirigir a la página que originó la navegación al login
        setUsrLogeado(usr);
        navegar(primerSegmento, { replace: true });
        return;
      }

      dialogo.alert(
        'El usuario no tiene ningún rol asignado. ' +
        'Consulte con el administrador.', { title: 'DNI: ' + dni });
    } catch (error) {
      let msg = 'Error inesperado. Contacte con el administrador';

      if (error instanceof AxiosError) {
        msg = error.response?.data || msg;
      } else {
        console.log(error);
      }

      dialogo.alert(msg, { title: 'DNI: ' + dni });
    }
  }

  return (
    <SignInContainer direction="column" justifyContent="space-between">
      <ColorModeSelect sx={{ position: 'fixed', top: '1rem', right: '1rem' }} />
      <Card variant="outlined">
        <SitemarkIcon />
        <Typography
          component="h1"
          variant="h4"
          sx={{ width: '100%', fontSize: 'clamp(2rem, 10vw, 2.15rem)' }}
        >
          Login
        </Typography>
        <Box
          component="form"
          onSubmit={handleSubmit}
          noValidate
          sx={{
            display: 'flex',
            flexDirection: 'column',
            width: '100%',
            gap: 2,
          }}
        >
          <FormControl>
            <FormLabel htmlFor="dni">DNI</FormLabel>
            <TextField
              id="dni"
              type="dni"
              name="dni"
              autoFocus
              required
              fullWidth
              variant="outlined"
            />
          </FormControl>
          <FormControl>
            <FormLabel htmlFor="password">Password</FormLabel>
            <TextField
              name="password"
              placeholder="••••••"
              type="password"
              id="password"
              required
              fullWidth
              variant="outlined"
            />
          </FormControl>
          <Button
            type="submit"
            fullWidth
            variant="contained"
          >
            INICIAR
          </Button>
        </Box>
      </Card>
    </SignInContainer>
  );
}

