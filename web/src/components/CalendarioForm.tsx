import * as React from 'react';
import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import FormGroup from '@mui/material/FormGroup';
import Grid from '@mui/material/Grid';
import Stack from '@mui/material/Stack';
import TextField from '@mui/material/TextField';
import ArrowBackIcon from '@mui/icons-material/ArrowBack';
import { useNavigate } from 'react-router';
import { FULL_HEIGHT_WIDTH } from '../context/DashboardSidebarContext';
import { useIsMounted } from '../hooks/useComponentMounted';
import { Calendario } from '../modelos/calendario';

export interface CalendarioFormState {
  values: Partial<Calendario>;
  errors: Partial<Record<keyof CalendarioFormState['values'], string>>;
}

export type FormFieldValue = string | number | null;

export interface CalendarioFormProps {
  formState: CalendarioFormState;
  onFieldChange: (
    name: keyof CalendarioFormState['values'],
    value: FormFieldValue,
  ) => void;
  onSubmit: (formValues: Partial<CalendarioFormState['values']>) => Promise<void>;
  onReset?: () => void;
  submitButtonLabel: string;
  backButtonPath?: string;
}

export type ValidationResult = {
  issues: { message: string; path: (keyof Calendario)[] }[]
};

export function validaCalendario(calendario: Partial<Calendario>): ValidationResult {
  const issues: ValidationResult['issues'] = [];

  if (!calendario.nombre?.trim()) {
    issues.push({ message: 'El nombre es requerido', path: ['nombre'] });
  } else if (calendario.nombre.length > 50) {
    issues.push({ message: 'El nombre no puede exceder los 50 caracteres', path: ['nombre'] });
  }

  if (!calendario.descripcion?.trim()) {
    issues.push({ message: 'La descripción es requerida', path: ['descripcion'] });
  } else if (calendario.descripcion.length > 200) {
    issues.push({ message: 'La descripción no puede exceder los 200 caracteres', path: ['descripcion'] });
  }

  return { issues };
}

export default function CalendarioForm(props: CalendarioFormProps) {
  const {
    formState,
    onFieldChange,
    onSubmit,
    onReset,
    submitButtonLabel,
    backButtonPath,
  } = props;

  const navigate = useNavigate();
  const isMounted = useIsMounted();

  const { values: formValues, errors: formErrors } = formState;

  const [isSubmitting, setIsSubmitting] = React.useState(false);

  const handleSubmit = React.useCallback(
    async (event: React.FormEvent<HTMLFormElement>) => {
      event.preventDefault();
      setIsSubmitting(true);

      try {
        await onSubmit(formValues);
      } finally {
        if (isMounted.current) {
          setIsSubmitting(false);
        }
      }
    },
    [formValues, onSubmit, isMounted],
  );

  const handleTextFieldChange = React.useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      onFieldChange(
        event.target.name as keyof CalendarioFormState['values'],
        event.target.value,
      );
    },
    [onFieldChange],
  );

  const handleBack = React.useCallback(() => {
    navigate(backButtonPath ?? '/calendarios');
  }, [navigate, backButtonPath]);

  return (
    <Box
      component="form"
      onSubmit={handleSubmit}
      noValidate
      autoComplete="off"
      onReset={onReset}
      sx={FULL_HEIGHT_WIDTH}
    >
      <FormGroup>
        <Grid container spacing={2} sx={{ mt: 2, mb: 4, width: '100%' }}>
          <Grid size={{ xs: 12 }}>
            <TextField
              value={formValues.nombre ?? ''}
              onChange={handleTextFieldChange}
              name="nombre"
              label="Nombre"
              error={!!formErrors.nombre}
              helperText={formErrors.nombre ?? ' '}
              fullWidth
              required
            />
          </Grid>
          <Grid size={{ xs: 12 }}>
            <TextField
              value={formValues.descripcion ?? ''}
              onChange={handleTextFieldChange}
              name="descripcion"
              label="Descripción"
              error={!!formErrors.descripcion}
              helperText={formErrors.descripcion ?? ' '}
              fullWidth
              required
            />
          </Grid>
        </Grid>
      </FormGroup>
      <Stack direction="row" spacing={2} justifyContent="space-between">
        <Button variant="contained" startIcon={<ArrowBackIcon />}
          onClick={handleBack}>
          VOLVER
        </Button>
        <Button type="submit" variant="contained" size="large"
          loading={isSubmitting}>
          {submitButtonLabel}
        </Button>
      </Stack>
    </Box>
  );
}