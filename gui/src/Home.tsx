import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog"
import { useNavigate } from 'react-router-dom';

import AppBar from "@mui/material/AppBar";
import Box from "@mui/material/Box";
import Button from "@mui/material/Button";
import Stack from "@mui/material/Stack";
import Toolbar from "@mui/material/Toolbar";
import Typography from "@mui/material/Typography";

import "./App.css";

function App() {
  const navigate = useNavigate();

  async function openDestination() {
    const path = await open();
    if (!path) {
      return;
    }
    await invoke("set_destination_path", { path });
    navigate("/destination");
  }

  return (
    <Box>
      <AppBar position="fixed">
        <Toolbar>
          <Typography variant="h6" component="div">
            Home
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
	<Button onClick={openDestination} variant="contained">Open destination</Button>
      </Stack>
    </Box>
  );
}

export default App;
