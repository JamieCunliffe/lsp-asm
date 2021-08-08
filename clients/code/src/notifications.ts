import * as lc from 'vscode-languageclient/node';
import { Position, Range, TextEditor, TextEditorRevealType, Uri, window} from 'vscode';


export async function openLoc(location: lc.Location) {
  const uri = location.uri;
  const range = location.range;

  await window.showTextDocument(Uri.parse(uri)).then(function(editor: TextEditor) {
    let r = new Range(new Position(range.start.line, range.start.character), new Position(range.end.line, range.end.character));
    editor.revealRange(r, TextEditorRevealType.InCenter);
  })
}
