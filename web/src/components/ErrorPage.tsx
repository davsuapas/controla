import React from 'react';
import {
  Box,
  Typography,
  Button,
  Container,
  Paper,
  useTheme
} from '@mui/material';
import { Home, Refresh, Warning } from '@mui/icons-material';
import { useNavigate, useRouteError } from 'react-router';

interface ErrorPageProps {
  type?: '404' | '500' | 'generic';
  message?: string;
}

export const ErrorPage: React.FC<ErrorPageProps> = ({ type, message }) => {
  const theme = useTheme();
  const navigate = useNavigate();
  const error = useRouteError() as any;

  // Determinar el tipo de error basado en el error de ruta o props
  const errorType = type ||
    (error?.status === 404 ? '404' :
      error?.status === 500 ? '500' : 'generic');

  const errorMessage = message ||
    error?.data?.message ||
    error?.message ||
    'Ha ocurrido un error inesperado';

  const getErrorConfig = () => {
    switch (errorType) {
      case '404':
        return {
          title: 'Página no encontrada',
          description: 'La página que estás buscando no existe o ha sido movida.',
          icon: '🔍',
          color: theme.palette.warning.main
        };
      case '500':
        return {
          title: 'Error del servidor',
          description: 'Algo salió mal en nuestro servidor. Por favor, intenta nuevamente.',
          icon: '⚙️',
          color: theme.palette.error.main
        };
      default:
        return {
          title: 'Algo salió mal',
          description: 'Ha ocurrido un error inesperado. Por favor, intenta nuevamente.',
          icon: '⚠️',
          color: theme.palette.error.main
        };
    }
  };

  const config = getErrorConfig();

  const handleGoHome = () => {
    navigate('/');
  };

  const handleRetry = () => {
    window.location.reload();
  };

  const handleGoBack = () => {
    navigate(-1);
  };

  return (
    <Container maxWidth="md">
      <Box
        sx={{
          minHeight: '100vh',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          py: 4
        }}
      >
        <Paper
          elevation={0}
          sx={{
            p: 6,
            textAlign: 'center',
            border: `2px dashed ${config.color}20`,
            backgroundColor: `${config.color}08`,
            borderRadius: 3
          }}
        >
          <Box
            sx={{
              fontSize: '4rem',
              mb: 2
            }}
          >
            {config.icon}
          </Box>

          <Warning
            sx={{
              fontSize: 60,
              color: config.color,
              mb: 3
            }}
          />

          <Typography
            variant="h3"
            component="h1"
            gutterBottom
            sx={{
              fontWeight: 'bold',
              color: config.color,
              mb: 2
            }}
          >
            {config.title}
          </Typography>

          <Typography
            variant="h6"
            color="text.secondary"
            sx={{ mb: 3 }}
          >
            {config.description}
          </Typography>

          {errorMessage && (
            <Paper
              variant="outlined"
              sx={{
                p: 2,
                mb: 4,
                backgroundColor: 'background.default',
                maxWidth: 400,
                mx: 'auto'
              }}
            >
              <Typography
                variant="body2"
                fontFamily="monospace"
                color="text.secondary"
              >
                {errorMessage}
              </Typography>
            </Paper>
          )}

          <Box
            sx={{
              display: 'flex',
              gap: 2,
              justifyContent: 'center',
              flexWrap: 'wrap'
            }}
          >
            <Button
              variant="contained"
              startIcon={<Home />}
              onClick={handleGoHome}
              size="large"
            >
              Ir al Inicio
            </Button>

            <Button
              variant="outlined"
              startIcon={<Refresh />}
              onClick={handleRetry}
              size="large"
            >
              Reintentar
            </Button>

            <Button
              variant="text"
              onClick={handleGoBack}
              size="large"
            >
              Volver Atrás
            </Button>
          </Box>
        </Paper>
      </Box>
    </Container>
  );
};

export default ErrorPage;