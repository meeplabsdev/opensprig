if [ $# -eq 0 ]; then
    echo "Usage: $0 <input>"
    exit 1
fi

input="$1"
fps=$(ffprobe -v error -select_streams v:0 -show_entries stream=r_frame_rate -of default=noprint_wrappers=1:nokey=1 "$input" 2>/dev/null)

if [ -z "$fps" ] || [ "$fps" = "0/0" ]; then
    ffmpeg -i "$input" -vf "scale=160:128,transpose" -pix_fmt bgr565be -f rawvideo "${input}.bin"
else
    ffmpeg -i "$input" -vf "fps=20,scale=160:128,transpose" -pix_fmt bgr565be -f rawvideo "${input}.bin"
fi