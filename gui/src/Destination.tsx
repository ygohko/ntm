import { invoke } from "@tauri-apps/api/core";
import { useState } from 'react';

import AppBar from "@mui/material/AppBar";
import ArrowBackIcon from "@mui/icons-material/ArrowBack";
import Box from "@mui/material/Box";
import Button from "@mui/material/Button";
import Dialog from '@mui/material/Dialog';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';
import DialogContentText from '@mui/material/DialogContentText';
import DialogTitle from '@mui/material/DialogTitle';
import IconButton from "@mui/material/IconButton";
import Snackbar from '@mui/material/Snackbar';
import Stack from "@mui/material/Stack";
import Toolbar from "@mui/material/Toolbar";
import Typography from "@mui/material/Typography";

import "./App.css";

function App() {
  const [succeededSnackbarShown, setSucceededSnackbarShown] = useState(false);
  const [failedDialogShown, setFailedDialogShown] = useState(false);

  async function execute() {
    const result = await invoke("execute", {});
    if (result) {
      setSucceededSnackbarShown(true);
    } else {
      setFailedDialogShown(true);
    }
  }

  function hideSnackBar() {
    setSucceededSnackbarShown(false);
  }

  function hideDialog() {
    setFailedDialogShown(false);
  }

  return (
    <Box>
      <AppBar position="fixed">
        <Toolbar>
          <IconButton
            size="large"
            edge="start"
            color="inherit"
            aria-label="back"
            sx={{ mr: 2 }}
            onClick={() => window.history.back()}
          >
            <ArrowBackIcon />
          </IconButton>
          <Typography variant="h6" component="div">
            Destination
          </Typography>
        </Toolbar>
      </AppBar>

      <Stack
	direction="column"
	sx={{
	  justifyContent: "center",
	  alignItems: "center",
	  minHeight: "100svh",
	}}
      >
	<Button onClick={execute} variant="contained">Execute</Button>
      </Stack>
      <Snackbar
	open={succeededSnackbarShown}
        autoHideDuration={3000}
        onClose={hideSnackBar}
        message="Backup succeeded!"
      />
      <Dialog
        open={failedDialogShown}
        onClose={hideDialog}
        role="alertdialog"
      >
        <DialogTitle>Backup failed</DialogTitle>
        <DialogContent>
          <DialogContentText>
            Backing up to this destination is failed.
          </DialogContentText>
        </DialogContent>
        <DialogActions>
          <Button onClick={hideDialog}>OK</Button>
        </DialogActions>
      </Dialog>      
    </Box>
  );
}

export default App;
