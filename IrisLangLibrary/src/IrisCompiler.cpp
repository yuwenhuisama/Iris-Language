#include "IrisCompiler.h"
#include "IrisComponents/IrisStatements/IrisStatement.h"
#include "IrisUnil/IrisVirtualCodeFile.h"
#include <iostream>
#include <iomanip>
using namespace std;

char* g_pCurrentString = NULL;
int g_nCurrentStringLength = 0;
int g_nReadLength = 0;

#ifdef IR_DEBUG_PRINT
#include "IrisInstructorMaker.h"
#endif

IrisCompiler* IrisCompiler::s_pCompiler = nullptr;

IrisCompiler::IrisCompiler()
{
}

void IrisCompiler::Release()
{
	for (auto& pData : m_mpStatementInfos) {
		delete pData.second;
	}
}

void IrisCompiler::SetCurrentFile(const string& strFile) {
	m_strCurFileName = strFile;
	//m_nCurFileIndex = GetFileIndex(m_strCurFileName);
	_StringField::iterator iFileName;
	if ((iFileName = m_mpFiles.find(strFile)) == m_mpFiles.end()) {
		int nResult = m_mpFiles.size();
		m_mpFiles.insert(_StringPair(strFile, nResult));
		m_vcFiles.push_back(strFile);
	}
	_StatementList* pList = nullptr;
	decltype(m_mpStatementInfos)::iterator iList;
	if ((iList = m_mpStatementInfos.find(strFile)) == m_mpStatementInfos.end()) {
		m_pCurrentStatementInfo = new _StatementInfo();
		m_pCurrentStatementInfo->m_pFloatField = new _FloatField();
		m_pCurrentStatementInfo->m_pIdentifierField = new _StringField();
		m_pCurrentStatementInfo->m_pIntegerField = new _IntegerField();
		m_pCurrentStatementInfo->m_pStatementList = new _StatementList();
		m_pCurrentStatementInfo->m_pStringField = new _StringField();
		m_pCurrentStatementInfo->m_pFloatSpace = new _FloatSpace();
		m_pCurrentStatementInfo->m_pIntegerSpace = new _IntegerSpace();
		m_pCurrentStatementInfo->m_pStringSpace = new _StringSpace();
		m_pCurrentStatementInfo->m_pIdentifierSpace = new _StringSpace();

		m_pCurrentStatementInfo->m_pUniqueStringField = new _StringField();
		m_pCurrentStatementInfo->m_pUniqueStringSpace = new _StringSpace();

		m_pCurrentStatementInfo->m_pCodes = new _CodeVector();

		m_mpStatementInfos.insert(_StatementsPair(strFile, m_pCurrentStatementInfo));
		m_vcStatementInfos.push_back(m_pCurrentStatementInfo);
		m_nCurFileIndex = m_vcStatementInfos.size() - 1;
	}
	else {
		m_pCurrentStatementInfo = iList->second;
	}
}

bool IrisCompiler::LoadScript(const string& strFile) {

	extern char* g_pCurrentString;
	extern int g_nCurrentStringLength;
	extern int g_nReadLength;
	extern int yyparse(void);

	IrisVirtualCodeFile ivcfIRCFile;

	SetCurrentFile(strFile);
	auto pInfo = m_mpStatementInfos[strFile];

	if (!ivcfIRCFile.LoadFromFile(strFile, pInfo)) {
		string strLine;
		string strText;

		fstream fFile("sentence.utf8");

		//fFile.imbue(locale(locale::classic(), new codecvt_utf8<wchar_t>));
		//std::locale::global(std::locale(""));

		fFile.open(strFile, ios::in);
		if (!fFile) {
			fprintf(stderr, "%s not found.\n", strFile.c_str());
			//exit(1);
			return false;
		}

		while (getline(fFile, strLine)) {
			strText += strLine + "\n";
		}

		fFile.close();
		
		IrisCompiler::CurrentCompiler()->ResetLineNumber();

		g_pCurrentString = (char*)strText.c_str();
		g_nCurrentStringLength = strText.length();
		g_nReadLength = 0;

		if (yyparse()) {
			/* BUGBUG */
			//exit(1);
			return false;
		}
	}

	return true;
}

bool IrisCompiler::LoadScriptString(const string & strScript)
{
	extern char* g_pCurrentString;
	extern int g_nCurrentStringLength;
	extern int g_nReadLength;
	extern int yyparse(void);

	m_bEvalFlag = true;
	
	g_pCurrentString = (char*)strScript.c_str();
	g_nCurrentStringLength = strScript.length();
	g_nReadLength = 0;

	if (yyparse()) {
		/* BUGBUG */
		return false;
	}
	return true;
}

bool IrisCompiler::LoadScriptFromVirtualPathAndText(const string & strPath, const string & strScript)
{
	extern char* g_pCurrentString;
	extern int g_nCurrentStringLength;
	extern int g_nReadLength;
	extern int yyparse(void);

	SetCurrentFile(strPath);

	string strLine;
	string strText;

	fstream fFile("sentence.utf8");

	//fFile.imbue(locale(locale::classic(), new codecvt_utf8<wchar_t>));
	//std::locale::global(std::locale(""));

	g_pCurrentString = (char*)strScript.c_str();
	g_nCurrentStringLength = strScript.length();
	g_nReadLength = 0;

	if (yyparse()) {
		/* BUGBUG */
		//exit(1);
		return false;
	}
	return true;
}

bool IrisCompiler::HaveFileRequired(const string & strFile)
{
	return m_mpStatementInfos.find(strFile) != m_mpStatementInfos.end();
}

bool IrisCompiler::Generate()
{
	
	if (!m_bEvalFlag) {
		for (auto stmt : *m_pCurrentStatementInfo->m_pStatementList) {
			if (!stmt->Generate()) {
				return false;
			}
		}
		// Release
		for (auto stmt : *m_pCurrentStatementInfo->m_pStatementList) {
			delete stmt;
		}
		delete m_pCurrentStatementInfo->m_pStatementList;
		m_pCurrentStatementInfo->m_pStatementList = nullptr;

		IrisVirtualCodeFile ivcfIRCFile;
		auto& strCurFileName = GetCurrentFileName();
		auto pInfo = m_mpStatementInfos[strCurFileName];
		ivcfIRCFile.SaveToFile(strCurFileName, pInfo);
	}
	else {
		for (auto stmt : m_pEvalStatements) {
			if (!stmt->Generate()) {
				return false;
			}
		}
		// Release
		for (auto stat : m_pEvalStatements) {
			delete stat;
		}
		m_pEvalStatements.clear();
	}

	m_bEvalFlag = false;

#ifdef IR_DEBUG_PRINT

	!m_bEvalFlag ? OutputCode(*m_pCurrentStatementInfo->m_pCodes) : OutputCode(m_vcEvalCodes);

	cout << endl;
	cout << "--------Static Data---------" << endl;
	cout << "String Field:" << endl;
	for (auto str : *m_pCurrentStatementInfo->m_pStringSpace) {
#ifdef IR_USE_STL_STRING
		StringReplace(str, "\n", "\\n");
		StringReplace(str, "\t", "\\t");
		cout << setw(6) << str << "\t";
#else
		StringReplace((string&)(str.GetSTLString()), "\n", "\\n");
		StringReplace((string&)(str.GetSTLString()), "\t", "\\t");
		cout << setw(6) << str.GetSTLString() << "\t";
#endif // IR_USE_STL_STRING
	}
	cout << endl;

	cout << "UniqueString Field:" << endl;
	for (auto str : *m_pCurrentStatementInfo->m_pUniqueStringSpace) {
#ifdef IR_USE_STL_STRING
		StringReplace(str, "\n", "\\n");
		StringReplace(str, "\t", "\\t");
		cout << setw(6) << str << "\t";
#else
		StringReplace((string&)(str.GetSTLString()), "\n", "\\n");
		StringReplace((string&)(str.GetSTLString()), "\t", "\\t");
		cout << setw(6) << str.GetSTLString() << "\t";
#endif // IR_USE_STL_STRING
	}
	cout << endl;

	cout << "Identifier Field:" << endl;
	for (auto id : *m_pCurrentStatementInfo->m_pIdentifierSpace) {
#ifdef IR_USE_STL_STRING
		cout << setw(6) << id << "\t";
#else
		cout << setw(6) << id.GetSTLString() << "\t";
#endif // IR_USE_STL_STRING
	}
	cout << endl;

	cout << "Integer Field:" << endl;
	for (auto itg : *m_pCurrentStatementInfo->m_pIntegerSpace) {
		cout << setw(6) << itg << "\t";
	}
	cout << endl;

	cout << "Float Field" << endl;
	for (auto flt : *m_pCurrentStatementInfo->m_pFloatSpace) {
		cout << setw(6) << flt << "\t";
	}
	cout << endl;
#endif
	return true;
}

#ifdef IR_DEBUG_PRINT
void IrisCompiler::StringReplace(string& s1, const string& s2, const string& s3)
{
	string::size_type pos = 0;
	string::size_type a = s2.size();
	string::size_type b = s3.size();
	while ((pos = s1.find(s2, pos)) != string::npos)
	{
		s1.replace(pos, a, s3);
		pos += b;
	}
}
#endif


#ifdef IR_DEBUG_PRINT
void IrisCompiler::OutputCode(vector<IR_WORD>& vcVector)
{

	unsigned int nCodePointer = 0;
	unsigned int nCodeEnder = vcVector.size();

	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	IR_BYTE bInstructor = 0;
	IrisAM iaAM;
	while (nCodePointer != nCodeEnder) {
		nCodePointer += 1;
		bInstructor =  vcVector[nCodePointer] >> 8;
		switch (bInstructor)
		{
		case 0: // push_env
			cout << std::left << setw(15) << setfill(' ') << "push_env" << endl;
			break;
		case 1: // pop_env
			cout << std::left << setw(15) << "pop_env" << endl;
			break;
		case 2: // push
			cout << std::left << setw(15) << "push" << endl;
			break;
		case 3: // pop
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << std::left << setw(15) << "pop" << "\t" << pMaker->GetAMString(IrisAMType::Extends) << "[" << iaAM.m_dwIndex << "]" << endl;
			break;
		case 4: // cre_env
			cout << std::left << setw(15) << "cre_env" << endl;
			break;
		case 5: // load
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << std::left << setw(15) << "load" << "\t" << pMaker->GetAMString((IrisAMType)iaAM.m_bType) << "[" << iaAM.m_dwIndex << "]" << endl;
			break;
		case 6: // nol_call
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << std::left << setw(15) << "nol_call" << "\t" << pMaker->GetAMString(IrisAMType::Identifier) << "[" << iaAM.m_dwIndex << "]";
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << ", " << pMaker->GetAMString(IrisAMType::Extends) << "[" << iaAM.m_dwIndex << "]" << endl;
			break;
		case 7: // assign
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << std::left << setw(15) << "assign" << "\t" << pMaker->GetAMString((IrisAMType)iaAM.m_bType) << "[" << iaAM.m_dwIndex << "]" << endl;
			break;
		case 8:	// hid_call
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << std::left << setw(15) << "hid_call" << "\t" << pMaker->GetAMString(IrisAMType::Identifier) << "[" << iaAM.m_dwIndex << "]";
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << ", " << pMaker->GetAMString(IrisAMType::Extends) << "[" << iaAM.m_dwIndex << "]" << endl;
			break;
		case 9:  // set_fld
		{
			cout << std::left << setw(15) << "set_fld" << "\t";
			unsigned int nFieldCount = vcVector[nCodePointer] & 0x00FF;
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << pMaker->GetAMString(IrisAMType::Identifier) << "[" << iaAM.m_dwIndex << "]";
			for (unsigned int i = 1; i < nFieldCount; ++i) {
				iaAM = GetOneAM(vcVector, nCodePointer);
				cout << ", " << pMaker->GetAMString(IrisAMType::Identifier) << "[" << iaAM.m_dwIndex << "]";
			}
			cout << endl;
		}
			break;
		case 10: // clr_fld
			cout << std::left << setw(15) << "clr_fld" << endl;
			break;
		case 11: // fld_load
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << std::left << setw(15) << "fld_load" << "\t" << pMaker->GetAMString(IrisAMType::Constance) << "[" << iaAM.m_dwIndex << "]";
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << "," << pMaker->GetAMString(IrisAMType::Extends) << "[" << iaAM.m_dwIndex << "]" << endl;
			break;
		case 12: // load_nil
			cout << std::left << setw(15) << "fld_load" << endl;
			break;
		case 13: // load_true
			cout << std::left << setw(15) << "load_true" << endl;
			break;
		case 14: // load_false
			cout << std::left << setw(15) << "load_false" << endl;
			break;
		case 15: // load_self
			cout << std::left << setw(15) << "load_self" << endl;
			break;
		case 16: // imth_def
		{
			unsigned int nParameterCount = (vcVector[nCodePointer] & 0x00FF) - 4;
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << std::left << setw(15) << "imth_def" << "\t" << pMaker->GetAMString(IrisAMType::Identifier) << "[" << iaAM.m_dwIndex << "]";
			for (unsigned int i = 0; i < nParameterCount; ++i) {
				iaAM = GetOneAM(vcVector, nCodePointer);
				cout << ", " << pMaker->GetAMString(IrisAMType::Identifier) << "[" << iaAM.m_dwIndex << "]";
			}
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << ", " << pMaker->GetAMString(IrisAMType::Identifier) << "[" << iaAM.m_dwIndex << "]";
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << ", " << pMaker->GetAMString(IrisAMType::Extends) << "[" << iaAM.m_dwIndex << "]";
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << ", " << pMaker->GetAMString(IrisAMType::Extends) << "[" << iaAM.m_dwIndex << "]" << endl;
		}
			break;
		case 17: // cmth_def
		{
			unsigned int nParameterCount = (vcVector[nCodePointer] & 0x00FF) - 4;
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << std::left << setw(15) << "imth_def" << "\t" << pMaker->GetAMString(IrisAMType::Identifier) << "[" << iaAM.m_dwIndex << "]";
			for (unsigned int i = 0; i < nParameterCount; ++i) {
				iaAM = GetOneAM(vcVector, nCodePointer);
				cout << ", " << pMaker->GetAMString(IrisAMType::Identifier) << "[" << iaAM.m_dwIndex << "]";
			}
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << ", " << pMaker->GetAMString(IrisAMType::Identifier) << "[" << iaAM.m_dwIndex << "]";
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << ", " << pMaker->GetAMString(IrisAMType::Extends) << "[" << iaAM.m_dwIndex << "]";
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << ", " << pMaker->GetAMString(IrisAMType::Extends) << "[" << iaAM.m_dwIndex << "]" << endl;
		}
		break;
		case 18: // blk_def
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << std::left << setw(15) << "blk_def" << pMaker->GetAMString(IrisAMType::Extends) << "[" << iaAM.m_dwIndex << "]" << endl;
			break;
		case 19: // end_def
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << std::left << setw(15) << "end_def" << pMaker->GetAMString(IrisAMType::Extends) << "[" << iaAM.m_dwIndex << "]" << endl;
			break;
		case 20: // jfon
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << std::left << setw(15) << "jfon" << pMaker->GetAMString(IrisAMType::Extends) << "[" << iaAM.m_dwIndex << "]" << endl;
			break;
		case 21: // jmp
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << std::left << setw(15) << "jmp" << pMaker->GetAMString(IrisAMType::Extends) << "[" << iaAM.m_dwIndex << "]" << endl;
			break;
		case 22: // ini_tm
			cout << std::left << setw(15) << "ini_tm" << endl;
			break;
		case 23: // ini_cnt
			cout << std::left << setw(15) << "ini_cnt" << endl;
			break;
		case 24: // cmp_tac
			cout << std::left << setw(15) << "cmp_tac" << endl;
			break;
		case 25: // inc_cnt
			cout << std::left << setw(15) << "inc_cnt" << endl;
			break;
		case 26: // assign_log
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << std::left << setw(15) << "assign_log" << pMaker->GetAMString(IrisAMType::LocalValue) << "[" << iaAM.m_dwIndex << "]" << endl;
			break;
		case 27: // brk
			cout << std::left << setw(15) << "brk" << endl;
			break;
		case 28: // push_deep
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << std::left << setw(15) << "push_deep" << pMaker->GetAMString(IrisAMType::Extends) << "[" << iaAM.m_dwIndex << "]" << endl;
			break;
		case 29: // pop_deep 
			cout << std::left << setw(15) << "pop_deep" << endl;
			break;
		case 30: // rtn
			cout << std::left << setw(15) << "rtn" << endl;
			break;
		case 31: // ctn
			cout << std::left << setw(15) << "ctn" << endl;
			break;
		case 32: // assign_vsl
			cout << std::left << setw(15) << "assign_vsl" << endl;
			break;
		case 33: // assign_iter
			cout << std::left << setw(15) << "assign_iter" << endl;
			break;
		case 34: // load_iter
			cout << std::left << setw(15) << "load_iter" << endl;
			break;
		case 35: // jt
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << std::left << setw(15) << "jt" << pMaker->GetAMString(IrisAMType::Extends) << "[" << iaAM.m_dwIndex << "]" << endl;
			break;
		case 36: // assign_cmp
			cout << std::left << setw(15) << "assign_cmp" << endl;
			break;
		case 37: // cmp_cmp
			cout << std::left << setw(15) << "cmp_cmp" << endl;
			break;
		case 38: // cre_cenv
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << std::left << setw(15) << "cre_cenv" <<  pMaker->GetAMString(IrisAMType::Extends) << "[" << iaAM.m_dwIndex << "]" << endl;
			break;
		case 39: // def_cls
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << std::left << setw(15) << "def_cls";
			cout << pMaker->GetAMString(IrisAMType::Identifier) << "[" << iaAM.m_dwIndex << "]";
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << "," << pMaker->GetAMString(IrisAMType::Extends) << "[" << iaAM.m_dwIndex << "]" << endl;
			break;
		case 40: // add_ext
			cout << std::left << setw(15) << "add_ext" << endl;
			break;
		case 41: // add_mld
			cout << std::left << setw(15) << "add_mld" << endl;
			break;
		case 42: // add_inf
			cout << std::left << setw(15) << "add_inf" << endl;
			break;
		case 43: // push_cnt
			cout << std::left << setw(15) << "push_cnt" << endl;
			break;
		case 44: // push_tim
			cout << std::left << setw(15) << "push_tim" << endl;
			break;
		case 45: // pop_cnt
			cout << std::left << setw(15) << "pop_cnt";
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << pMaker->GetAMString(IrisAMType::Identifier) << "[" << iaAM.m_dwIndex << "]" << endl;
			break;
		case 46: // pop_tim
			cout << std::left << setw(15) << "pop_tim";
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << pMaker->GetAMString(IrisAMType::Identifier) << "[" << iaAM.m_dwIndex << "]" << endl;
			break;
		case 47: // push_unim
			cout << std::left << setw(15) << "push_unim" << endl;
			break;
		case 48: // pop_unim
			cout << std::left << setw(15) << "pop_unim";
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << pMaker->GetAMString(IrisAMType::Identifier) << "[" << iaAM.m_dwIndex << "]" << endl;
			break;
		case 49: // push_vsl
			cout << std::left << setw(15) << "push_vsl" << endl;
			break;
		case 50: // pop_vsl
			cout << std::left << setw(15) << "pop_vsl";
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << pMaker->GetAMString(IrisAMType::Identifier) << "[" << iaAM.m_dwIndex << "]" << endl;
			break;
		case 51: // push_iter
			cout << std::left << setw(15) << "push_iter" << endl;
			break;
		case 52: // pop_iter
			cout << std::left << setw(15) << "pop_iter";
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << pMaker->GetAMString(IrisAMType::Identifier) << "[" << iaAM.m_dwIndex << "]" << endl;
			break;
		case 53: // str_def
			cout << std::left << setw(15) << "str_def";
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << pMaker->GetAMString(IrisAMType::Identifier) << "[" << iaAM.m_dwIndex << "]";
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << ", " << pMaker->GetAMString(IrisAMType::Identifier) << "[" << iaAM.m_dwIndex << "]";
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << ", " << pMaker->GetAMString(IrisAMType::Extends) << "[" << iaAM.m_dwIndex << "]";
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << ", " << pMaker->GetAMString(IrisAMType::Extends) << "[" << iaAM.m_dwIndex << "]" << endl;
			break;
		case 54: // gtr_def
			cout << std::left << setw(15) << "gtr_def";
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << pMaker->GetAMString(IrisAMType::Identifier) << "[" << iaAM.m_dwIndex << "]";
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << ", " << pMaker->GetAMString(IrisAMType::Extends) << "[" << iaAM.m_dwIndex << "]";
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << ", " << pMaker->GetAMString(IrisAMType::Extends) << "[" << iaAM.m_dwIndex << "]" << endl;
			break;
		case 55: // gstr_def
			cout << std::left << setw(15) << "gstr_def";
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << pMaker->GetAMString(IrisAMType::Identifier) << "[" << iaAM.m_dwIndex << endl;
			break;
		case 56: // set_auth
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << std::left << setw(15) << "set_auth";
			cout << pMaker->GetAMString(IrisAMType::Identifier) << "[" << iaAM.m_dwIndex << "]";
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << "," << pMaker->GetAMString(IrisAMType::Extends) << "[" << iaAM.m_dwIndex << "]";
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << "," << pMaker->GetAMString(IrisAMType::Extends) << "[" << iaAM.m_dwIndex << "]";
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << "," << pMaker->GetAMString(IrisAMType::Extends) << "[" << iaAM.m_dwIndex << "]" << endl;
			break;
		case 57: // def_mld
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << std::left << setw(15) << "def_mld";
			cout << pMaker->GetAMString(IrisAMType::Identifier) << "[" << iaAM.m_dwIndex << "]";
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << "," << pMaker->GetAMString(IrisAMType::Extends) << "[" << iaAM.m_dwIndex << "]" << endl;
			break;
		case 58: // def_inf
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << std::left << setw(15) << "def_inf";
			cout << pMaker->GetAMString(IrisAMType::Identifier) << "[" << iaAM.m_dwIndex << "]";
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << "," << pMaker->GetAMString(IrisAMType::Extends) << "[" << iaAM.m_dwIndex << "]" << endl;
			break;
		case 59: // def_infs
		{
			unsigned int nParameterCount = (vcVector[nCodePointer] & 0x00FF) - 2;
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << std::left << setw(15) << "def_infs" << "\t" << pMaker->GetAMString(IrisAMType::Identifier) << "[" << iaAM.m_dwIndex << "]";
			for (unsigned int i = 0; i < nParameterCount; ++i) {
				iaAM = GetOneAM(vcVector, nCodePointer);
				cout << ", " << pMaker->GetAMString(IrisAMType::Identifier) << "[" << iaAM.m_dwIndex << "]";
			}
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << ", " << pMaker->GetAMString(IrisAMType::Identifier) << "[" << iaAM.m_dwIndex << "]" << endl;
		}
			break;
		case 60: // cblk_def
		{
			unsigned int nParameterCount = (vcVector[nCodePointer] & 0x00FF) - 1;
			if (nParameterCount >= 1) {
				iaAM = GetOneAM(vcVector, nCodePointer);
				cout << std::left << setw(15) << "cblk_def" << "\t" << pMaker->GetAMString(IrisAMType::Identifier) << "[" << iaAM.m_dwIndex << "]";
				for (unsigned int i = 1; i < nParameterCount; ++i) {
					iaAM = GetOneAM(vcVector, nCodePointer);
					cout << ", " << pMaker->GetAMString(IrisAMType::Identifier) << "[" << iaAM.m_dwIndex << "]";
				}
			}
			else {
				cout << std::left << setw(15) << "cblk_def" << "\t";
				iaAM = GetOneAM(vcVector, nCodePointer);
				cout << pMaker->GetAMString(IrisAMType::Extends) << "[" << iaAM.m_dwIndex << "]" << endl;
			}
		}
			break;
		case 61: // blk
			cout << std::left << setw(15) << "blk" << endl;
			break;
		case 62: // cast
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << std::left << setw(15) << "cast" << pMaker->GetAMString(IrisAMType::Extends) << "[" << iaAM.m_dwIndex<< "]" << endl;
			break;
		case 63: // reg_irg 
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << std::left << setw(15) << "reg_irg" << pMaker->GetAMString(IrisAMType::Extends) << "[" << iaAM.m_dwIndex << "]";
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << ", " << pMaker->GetAMString(IrisAMType::Extends) << "[" << iaAM.m_dwIndex << "]" << endl;
			break;
		case 64: // ureg_irp
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << std::left << setw(15) << "ureg_irp" << pMaker->GetAMString(IrisAMType::Extends) << "[" << iaAM.m_dwIndex << "]" << endl;
			break;
		case 65: // assign_ir
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << std::left << setw(15) << "assign_ir" << pMaker->GetAMString(IrisAMType::LocalValue) << "[" << iaAM.m_dwIndex << "]" << endl;
			break;
		case 66: // grn
			cout << std::left << setw(15) << "grn" << endl;
			break;
		case 67: // spr
			iaAM = GetOneAM(vcVector, nCodePointer);
			cout << std::left << setw(15) << "spr" << pMaker->GetAMString(IrisAMType::Extends) << "[" << iaAM.m_dwIndex << "]" << endl;
			break;
		default:
			break;
		}
		++nCodePointer;
	}
}

IrisAM IrisCompiler::GetOneAM(vector<IR_WORD>& vcVector, unsigned int & nCodePointer)
{
	IR_BYTE bType = (IR_BYTE)vcVector[++nCodePointer];
	IR_WORD wLow = vcVector[++nCodePointer];
	IR_WORD wHigh = vcVector[++nCodePointer];
	return IrisAM(bType, (((0x00000000) | wHigh) << 16) | wLow);
}
#endif

unsigned int IrisCompiler::GetStringIndex(const string & strString, unsigned int nFileIndex)
{
	auto pData = m_vcStatementInfos[nFileIndex];
	_StringField::iterator iString;
	if ((iString = pData->m_pStringField->find(strString)) == pData->m_pStringField->end()) {
		int nResult = pData->m_pStringField->size();
		pData->m_pStringField->insert(_StringPair(strString, nResult));
		pData->m_pStringSpace->push_back(strString);
		return nResult;
	}
	else {
		return iString->second; 
	}
}

unsigned int IrisCompiler::GetIntegerIndex(int nInteger, unsigned int nFileIndex)
{
	auto pData = m_vcStatementInfos[nFileIndex];
	_IntegerField::iterator iInteger;
	if ((iInteger = pData->m_pIntegerField->find(nInteger)) == pData->m_pIntegerField->end()) {
		int nResult = pData->m_pIntegerField->size();
		pData->m_pIntegerField->insert(_IntegerPair(nInteger, nResult));
		pData->m_pIntegerSpace->push_back(nInteger);
		return nResult;
	}
	else {
		return iInteger->second;
	}
}

unsigned int IrisCompiler::GetFloatIndex(double dFloat, unsigned int nFileIndex)
{
	auto pData = m_vcStatementInfos[nFileIndex];
	_FloatField::iterator iFloat;
	if ((iFloat = pData->m_pFloatField->find(dFloat)) == pData->m_pFloatField->end()) {
		int nResult = pData->m_pFloatField->size();
		pData->m_pFloatField->insert(_FloatPair(dFloat, nResult));
		pData->m_pFloatSpace->push_back(dFloat);
		return nResult;
	}
	else {
		return iFloat->second;
	}
}

unsigned int IrisCompiler::GetIdentifierIndex(const string & strIdentifier, unsigned int nFileIndex)
{
	auto pData = m_vcStatementInfos[nFileIndex];
	_StringField::iterator iIdentifier;
	if ((iIdentifier = pData->m_pIdentifierField->find(strIdentifier)) == pData->m_pIdentifierField->end()) {
		int nResult = pData->m_pIdentifierField->size();
		pData->m_pIdentifierField->insert(_StringPair(strIdentifier, nResult));
		pData->m_pIdentifierSpace->push_back(strIdentifier);
		return nResult;
	}
	else {
		return iIdentifier->second;
	}
}

unsigned int IrisCompiler::GetUniqueStringIndex(const string & strString, unsigned int nFileIndex)
{
	auto pData = m_vcStatementInfos[nFileIndex];
	_StringField::iterator iString;
	if ((iString = pData->m_pUniqueStringField->find(strString)) == pData->m_pUniqueStringField->end()) {
		int nResult = pData->m_pUniqueStringField->size();
		pData->m_pUniqueStringField->insert(_StringPair(strString, nResult));
		pData->m_pUniqueStringSpace->push_back(strString);
		return nResult;
	}
	else {
		return iString->second;
	}
}

//unsigned int IrisCompiler::GetFileIndex(const string & strFileName)
//{
//	_StringField::iterator iFileName;
//	if ((iFileName = m_mpFiles.find(strFileName)) == m_mpFiles.end()) {
//		int nResult = m_mpFiles.size();
//		m_mpFiles.insert(_StringPair(strFileName, nResult));
//		m_vcFiles.push_back(strFileName);
//		return nResult;
//	}
//	else {
//		return iFileName->second;
//	}
//}

IrisCompiler::~IrisCompiler()
{
}

bool IrisCompiler::UpperWithLoop()
{
	auto riIter = m_skUpperStack.rbegin();
	for (; riIter != m_skUpperStack.rend(); ++riIter) {
		if (*riIter == UpperType::Loop) {
			return true;
		}
	}
	return false;
}

bool IrisCompiler::UpperWithMethod()
{
	auto riIter = m_skUpperStack.rbegin();
	for (; riIter != m_skUpperStack.rend(); ++riIter) {
		if (*riIter == UpperType::Method) {
			return true;
		}
	}
	return false;
}
