from http.server import BaseHTTPRequestHandler,HTTPServer
from socketserver import ThreadingMixIn
import threading
import subprocess
import urllib.parse

# todo: factor out common server stuff

# todo: these should probably have limited
# access to files, so something like only
# uploads dir may be good.
# then there is slight problem about
# possibility to optimize theme files
# for example (which should be done first,
# but it'd be convenient to reuse this.)
# Maybe allow to mount a theme path

# Collecting args, stripping quotes string for
# it to work with subprocess.Popen
# Assuming only single quoted strings
def append_args(cmd_list, cmd_args):
    in_string = False
    accum = ""
    for i in range(0, len(cmd_args) - 1):
        char = cmd_args[i]
        if (in_string):
            if (char == "'"):
                cmd_list.append(accum)
                accum = ""
                in_string = False
            else:
                accum = accum + char
        else:
            if (char == " "):
                if (accum != ""):
                    cmd_list.append(accum)
                    accum = ""
            elif (accum == "" and char == "'"):
                in_string = True
            else:
                accum = accum + char
    if (accum != ""):
        cmd_list.append(accum)
    return cmd_list


class Handler(BaseHTTPRequestHandler):
    def do_POST(self):
        content_length = int(self.headers['Content-Length'])
        cmd_args = self.rfile.read(content_length).decode('utf-8')
        if len(cmd_args) > 0:
            # todo: Does this make config.yml end up in uploads?
            cmd_list = append_args([
                "image_optim",
                "--config-paths",
                "/opt/config.yml"
                ], cmd_args)
            CmdOut = subprocess.Popen(cmd_list)
            (stdout,stderr) = CmdOut.communicate()
            print(stdout)
            print(stderr)
        self.send_response(200)
        self.send_header("Content-type", "text/plain")
        self.end_headers()
        self.wfile.write("ok".encode('utf-8'))

    #def log_message(self, format, *args):
        # suppress logging per request
        #return

class ThreadingSimpleServer(ThreadingMixIn, HTTPServer):
    pass

if __name__ == '__main__':
    print('Image opti server starts')
    httpd = ThreadingSimpleServer(('0.0.0.0', 8971), Handler)
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        pass
    httpd.server_close()
    print('Image opti server stops')
