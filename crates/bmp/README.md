# BMP: Microsoft Windows Bitmap Image

各ピクセルの画素値をそのまま並べた画像フォーマット。

- 仕様書: 検索したがみあたらず

# ファイル構造

[英語版wikipedia](https://en.wikipedia.org/wiki/BMP_file_format)に乗っているメモリレイアウトの表がわかりやすくて良い。

## BitmapFileHeader

[win32API内での定義](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-bitmapfileheader)

ファイルのメタ情報。ファイルの読み込み時のみ必要で表示には必要ない。

## BitmapV5Header

[win32API内での定義](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-bitmapv5header)

画像に関するメタ情報。画像サイズやピクセルフォーマットなど。

## 画像データ

各ピクセルのRGB値が並んでいる。データの順序に注意。

- BGRの順番で並んでいる
- 下から上、左から右へと並んでいる (S字を逆順になぞる)

# 読み込みの流れ

1. BitmapFileHeader構造体を読み込む

   画像データの開始位置(ファイル先頭からの相対オフセット)がbyte_offsetに入っている

2. BitmapV5Header構造体を読み込む

   画像の幅、高さ、ピクセルデータのビット数が入っている。

3. 画像データを読み込む

   ピクセルデータの順序に注意して読み込む