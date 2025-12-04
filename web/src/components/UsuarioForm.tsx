import * as React from 'react';
import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import Checkbox from '@mui/material/Checkbox';
import FormControl from '@mui/material/FormControl';
import FormControlLabel from '@mui/material/FormControlLabel';
import FormGroup from '@mui/material/FormGroup';
import FormHelperText from '@mui/material/FormHelperText';
import Grid from '@mui/material/Grid';
import InputLabel from '@mui/material/InputLabel';
import MenuItem from '@mui/material/MenuItem';
import Select, { SelectChangeEvent, SelectProps } from '@mui/material/Select';
import Stack from '@mui/material/Stack';
import TextField from '@mui/material/TextField';
import ArrowBackIcon from '@mui/icons-material/ArrowBack';
import { useNavigate } from 'react-router';
import dayjs from 'dayjs';
import { nombresRoles, nombresTodosRoles, Rol, Usuario } from '../modelos/usuarios';
import OutlinedInput from '@mui/material/OutlinedInput';
import Chip from '@mui/material/Chip';
import { useTheme } from '@mui/material/styles';
import { FULL_HEIGHT_WIDTH } from '../context/DashboardSidebarContext';
import { useIsMounted } from '../hooks/useComponentMounted';

export interface UsuarioFormState {
  values: Partial<Usuario>;
  errors: Partial<Record<keyof UsuarioFormState['values'], string>>;
}

export enum Presentacion {
  FULL = 0,
  SOLO_PASSWORD = 1,
  SOLO_PASSWORD_SIN_VOLVER = 2,
  SIN_PASSWORD = 3,
}

export type FormFieldValue = string | string[] | number |
  boolean | dayjs.Dayjs | null;

// Parámetros del formulario
export interface UsuarioFormProps {
  formState: UsuarioFormState;
  onFieldChange: (
    name: keyof UsuarioFormState['values'],
    value: FormFieldValue,
  ) => void;
  onSubmit: (formValues: Partial<UsuarioFormState['values']>) => Promise<void>;
  onReset?: (formValues: Partial<UsuarioFormState['values']>) => void;
  submitButtonLabel: string;
  backButtonPath?: string;
  presentacion: Presentacion;
}

export type ValidationResult = {
  issues: { message: string; path: (keyof Usuario)[] }[]
};

export function concatenarValidaciones(
  ...results: ValidationResult[]): ValidationResult {
  const todasLasIssues: { message: string; path: (keyof Usuario)[] }[] = [];

  for (const result of results) {
    todasLasIssues.push(...result.issues);
  }

  return { issues: todasLasIssues };
}

// Valida los datos de la password de un usuario
export function validaUsuarioPass(
  usuario: Partial<Usuario>): ValidationResult {
  let issues: ValidationResult['issues'] = [];

  if (!usuario.password) {
    issues = [
      ...issues, {
        message: 'La password no puede estar vacía', path: ['password']
      }
    ];
  }

  if (usuario.password !== usuario.passConfirm) {
    issues = [
      ...issues, {
        message:
          'La password y la confirmación deben ser iguales',
        path: ['passConfirm']
      }
    ];
  }

  return { issues };
}

// Valida los datos de un usuario
export function validaUsuario(usuario: Partial<Usuario>): ValidationResult {
  let issues: ValidationResult['issues'] = [];

  if (!usuario.nombre) {
    issues = [
      ...issues, { message: 'El nombre es requerido', path: ['nombre'] }
    ];
  }

  if (!usuario.dni) {
    issues = [
      ...issues, { message: 'El DNI es requerido', path: ['dni'] }
    ];
  }

  if (!usuario.email || !validarEmail(usuario.email)) {
    issues = [
      ...issues, { message: 'El email no es correcto', path: ['email'] }
    ];
  }

  if (!usuario.primerApellido) {
    issues = [
      ...issues, {
        message: 'El primer apellido es requerido', path: ['primerApellido']
      }
    ];
  }

  if (!usuario.segundoApellido) {
    issues = [
      ...issues, {
        message: 'El segundo apellido es requerido', path: ['segundoApellido']
      }
    ];
  }

  if (!usuario.roles || usuario.roles.length == 0) {
    issues = [
      ...issues, {
        message: 'Debe asignar por lo menos un rol', path: ['roles']
      }
    ];
  }

  return { issues };
}

// Valida un email
function validarEmail(email: string): boolean {
  // Expresión regular para validar emails
  const regexEmail = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

  // Validaciones adicionales
  if (typeof email !== 'string') {
    return false;
  }

  if (email.length > 254) { // Longitud máxima según RFC 5321
    return false;
  }

  // Verificar que no empiece o termine con punto o guion
  if (email.startsWith('.') || email.startsWith('-') ||
    email.endsWith('.') || email.endsWith('-')) {
    return false;
  }

  // Verificar que tenga exactamente un símbolo @
  const arrobaCount = (email.match(/@/g) || []).length;
  if (arrobaCount !== 1) {
    return false;
  }

  // Dividir en usuario y dominio
  const partes = email.split('@');
  const usuario = partes[0];
  const dominio = partes[1];

  // Validar longitud del usuario (máximo 64 caracteres)
  if (usuario.length > 64 || usuario.length === 0) {
    return false;
  }

  // Validar que el dominio tenga al menos un punto
  if (!dominio.includes('.')) {
    return false;
  }

  // Validar que no haya puntos consecutivos
  if (email.includes('..')) {
    return false;
  }

  return regexEmail.test(email);
}

// Evento para la transformación de un campo
export function setPropGeneralesUsuario(
  name: keyof UsuarioFormState['values'],
  newValue: FormFieldValue): any {

  if (name === 'activo') {
    return newValue ? dayjs() : null;
  }

  if (name === 'roles') {
    return Array.isArray(newValue) ? newValue.map(Rol.desdeNombre) : [];
  }

  return newValue;
}

export default function UsuarioForm(props: UsuarioFormProps) {
  const {
    formState,
    onFieldChange,
    onSubmit,
    onReset,
    submitButtonLabel,
    backButtonPath,
    presentacion
  } = props;

  const theme = useTheme();
  const navigate = useNavigate();
  const isMounted = useIsMounted();

  const formValues = formState.values;
  const formErrors = formState.errors;

  const [isSubmitting, setIsSubmitting] = React.useState(false);

  // Maneja el envío del formulario
  const handleSubmit = React.useCallback(
    async (event: React.FormEvent<HTMLFormElement>) => {
      event.preventDefault();

      setIsSubmitting(true);
      try {
        await onSubmit(formValues);
      } finally {
        if (isMounted.current) {
          setIsSubmitting(false);
        };
      }
    },
    [formValues, onSubmit],
  );

  // Maneja el cambio de un campo text
  const handleTextFieldChange = React.useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      onFieldChange(
        event.target.name as keyof UsuarioFormState['values'],
        event.target.value,
      );
    },
    [onFieldChange],
  );

  // Maneja el cambio de un campo checkbox
  const handleCheckboxFieldChange = React.useCallback(
    (event: React.ChangeEvent<HTMLInputElement>, checked: boolean) => {
      onFieldChange(
        event.target.name as keyof UsuarioFormState['values'], checked);
    },
    [onFieldChange],
  );

  // Maneja el cambio de un campo select
  const handleSelectFieldChange = React.useCallback(
    (event: SelectChangeEvent) => {
      onFieldChange(
        event.target.name as keyof UsuarioFormState['values'],
        event.target.value,
      );
    },
    [onFieldChange],
  );

  const handleReset = React.useCallback(() => {
    if (onReset) {
      onReset(formValues);
    }
  }, [formValues, onReset]);

  // Botón de regreso
  const handleBack = React.useCallback(() => {
    navigate(backButtonPath ?? '/usuarios');
  }, [navigate, backButtonPath]);

  function estilosParaRoles(nombre: string) {
    const nombres = nombresRoles(formValues?.roles ?? []);

    return {
      fontWeight: nombres && nombres.includes(nombre)
        ? theme.typography.fontWeightMedium
        : theme.typography.fontWeightRegular,
    };
  }

  return (
    <Box
      component="form"
      onSubmit={handleSubmit}
      noValidate
      autoComplete="off"
      onReset={handleReset}
      sx={FULL_HEIGHT_WIDTH}
    >
      <FormGroup>
        <Grid container spacing={2} sx={{ mb: 2, width: '100%' }}>
          {(presentacion === Presentacion.FULL ||
            presentacion === Presentacion.SIN_PASSWORD) && (
              <>
                <Grid size={{ xs: 12, sm: 6 }} sx={{ display: 'flex' }}>
                  <TextField
                    value={formValues.dni ?? ''}
                    onChange={handleTextFieldChange}
                    name="dni"
                    label="DNI"
                    error={!!formErrors.dni}
                    helperText={formErrors.dni ?? ' '}
                    fullWidth
                  />
                </Grid>
                <Grid size={{ xs: 12, sm: 6 }} sx={{ display: 'flex' }}>
                  <TextField
                    value={formValues.nombre ?? ''}
                    onChange={handleTextFieldChange}
                    name="nombre"
                    label="Nombre"
                    error={!!formErrors.nombre}
                    helperText={formErrors.nombre ?? ' '}
                    fullWidth
                  />
                </Grid>
                <Grid size={{ xs: 12, sm: 6 }} sx={{ display: 'flex' }}>
                  <TextField
                    value={formValues.primerApellido ?? ''}
                    onChange={handleTextFieldChange}
                    name="primerApellido"
                    label="Primer apellido"
                    error={!!formErrors.primerApellido}
                    helperText={formErrors.primerApellido ?? ' '}
                    fullWidth
                  />
                </Grid>
                <Grid size={{ xs: 12, sm: 6 }} sx={{ display: 'flex' }}>
                  <TextField
                    value={formValues.segundoApellido ?? ''}
                    onChange={handleTextFieldChange}
                    name="segundoApellido"
                    label="Segundo apellido"
                    error={!!formErrors.segundoApellido}
                    helperText={formErrors.segundoApellido ?? ' '}
                    fullWidth
                  />
                </Grid>
                <Grid size={{ xs: 12, sm: 6 }} sx={{ display: 'flex' }}>
                  <TextField
                    value={formValues.email ?? ''}
                    onChange={handleTextFieldChange}
                    name="email"
                    label="Email"
                    type="email"
                    error={!!formErrors.email}
                    helperText={formErrors.email ?? ' '}
                    fullWidth
                  />
                </Grid>
                <Grid size={{ xs: 12, sm: 6 }} sx={{ display: 'flex' }}>
                  <FormControl error={!!formErrors.roles} fullWidth>
                    <InputLabel>Roles</InputLabel>
                    <Select
                      name="roles"
                      multiple
                      value={nombresRoles(formValues.roles ?? [])}
                      onChange={handleSelectFieldChange as SelectProps['onChange']}
                      input={<OutlinedInput label="Roles" />}
                      renderValue={(selected) => (
                        <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 0.5 }}>
                          {selected.map((value) => (
                            <Chip label={value} />
                          ))}
                        </Box>
                      )}
                    >
                      {nombresTodosRoles().map((rol) => (
                        <MenuItem
                          key={rol}
                          value={rol}
                          style={estilosParaRoles(rol)}
                        >
                          {rol}
                        </MenuItem>
                      ))}
                    </Select>
                    <FormHelperText>{formErrors.roles ?? ' '}</FormHelperText>
                  </FormControl>
                </Grid>
                <Grid size={{ xs: 12, sm: 6 }} sx={{ display: 'flex' }}>
                  <FormControl>
                    <FormControlLabel
                      name="activo"
                      control={
                        <Checkbox
                          size="large"
                          checked={formValues.activo != null}
                          onChange={handleCheckboxFieldChange}
                        />
                      }
                      label="Activo"
                    />
                    <FormHelperText error={!!formErrors.activo}>
                      {formErrors.activo ?? ' '}
                    </FormHelperText>
                  </FormControl>
                </Grid>
              </>
            )}
        </Grid>
        {(presentacion === Presentacion.FULL ||
          presentacion === Presentacion.SOLO_PASSWORD ||
          presentacion === Presentacion.SOLO_PASSWORD_SIN_VOLVER) && (
            <Grid container spacing={2} sx={{ mb: 2, width: '100%' }}>
              <Grid size={{ xs: 12, sm: 6 }} sx={{ display: 'flex' }}>
                <TextField
                  value={formValues.password ?? ''}
                  onChange={handleTextFieldChange}
                  name="password"
                  label="Password"
                  type="password"
                  error={!!formErrors.password}
                  helperText={formErrors.password ?? ' '}
                  fullWidth
                />
              </Grid>
              <Grid size={{ xs: 12, sm: 6 }} sx={{ display: 'flex' }}>
                <TextField
                  value={formValues.passConfirm ?? ''}
                  onChange={handleTextFieldChange}
                  name="passConfirm"
                  label="Confirmar password"
                  type="password"
                  error={!!formErrors.passConfirm}
                  helperText={formErrors.passConfirm ?? ' '}
                  fullWidth
                />
              </Grid>
            </Grid>
          )}
      </FormGroup>
      <Stack direction="row" spacing={2} justifyContent="space-between">
        {(presentacion !== Presentacion.SOLO_PASSWORD_SIN_VOLVER) && (
          <Button
            variant="contained"
            startIcon={<ArrowBackIcon />}
            onClick={handleBack}
          >
            VOLVER
          </Button>
        )}
        < Button
          type="submit"
          variant="contained"
          size="large"
          loading={isSubmitting}
        >
          {submitButtonLabel}
        </Button>
      </Stack>
    </Box >
  );
}
