#include "IrisComponents/IrisExpressions/IrisFieldExpression.h"
#include "IrisComponents/IrisParts/IrisFieldIdentifier.h"
#include "IrisInstructorMaker.h"
#include "IrisCompiler.h"
#include "IrisComponents/IrisExpressions/IrisIdentifierExpression.h"
#include "IrisUnil/IrisIdentifier.h"
#include "IrisCompiler.h"
#include "IrisFatalErrorHandler.h"


bool IrisFieldExpression::Generate()
{
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	pCompiler->SetLineNumber(m_nLineNumber);

  	if (m_pFieldIdentifier->m_pList) {
		list<IR_DWORD> lsFieldMembers;

		auto& lsField = m_pFieldIdentifier->m_pList->m_lsList;

		auto iFirstField = lsField.begin();

		lsFieldMembers.push_back(pCompiler->GetIdentifierIndex((*iFirstField)->GetIdentifierString(), pCompiler->GetCurrentFileIndex()));

		unsigned int nIndex = pCompiler->GetIdentifierIndex((*iFirstField)->GetIdentifierString(), pCompiler->GetCurrentFileIndex());

		switch ((*iFirstField)->GetType()) {
		case IrisIdentifierType::Constance:
			pMaker->load(IrisAMType::Constance, nIndex);
			break;
		case IrisIdentifierType::ClassVariable:
			pMaker->load(IrisAMType::ClassValue, nIndex);
			break;
		case IrisIdentifierType::GlobalVariable:
			pMaker->load(IrisAMType::GlobalValue, nIndex);
			break;
		case IrisIdentifierType::InstanceVariable:
			pMaker->load(IrisAMType::InstanceValue, nIndex);
			break;
		case IrisIdentifierType::LocalVariable:
			pMaker->load(IrisAMType::LocalValue, nIndex);
			break;
		}

		++iFirstField;

		for (; iFirstField != lsField.end(); ++iFirstField) {
			auto& pIdentifier = *iFirstField;
			lsFieldMembers.push_back(pCompiler->GetIdentifierIndex(pIdentifier->GetIdentifierString(), pCompiler->GetCurrentFileIndex()));
		}

		if (!lsFieldMembers.empty()) {
			pMaker->set_fld(lsFieldMembers);
		}
		pMaker->fld_load(pCompiler->GetIdentifierIndex(m_pFieldIdentifier->m_pIdentifier->GetIdentifierString(), pCompiler->GetCurrentFileIndex()), 1);
		if (!lsFieldMembers.empty()) {
			pMaker->clr_fld();
		}
	}
	else {
		pMaker->fld_load(pCompiler->GetIdentifierIndex(m_pFieldIdentifier->m_pIdentifier->GetIdentifierString(), pCompiler->GetCurrentFileIndex()), 0);
	}

	return true;
}

IrisFieldExpression::IrisFieldExpression(IrisFieldIdentifier* pFieldIdentifier) : m_pFieldIdentifier(pFieldIdentifier)
{
	m_bValidLeftValue = true;
}


IrisFieldExpression::~IrisFieldExpression()
{
	if (m_pFieldIdentifier) {
		delete m_pFieldIdentifier;
	}
}

bool IrisFieldExpression::Validate()
{
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();

	if (m_pFieldIdentifier->m_pList) {
		size_t nIndex = 0;
		if (!m_pFieldIdentifier->m_pList->Ergodic([&](IrisIdentifier*& pIdentifier)->bool {
			if (nIndex != 0) {
				if (pIdentifier->GetType() != IrisIdentifierType::Constance) {
					// ** Error **
					IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::IdenfierTypeIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "Identifier of " + pIdentifier->GetIdentifierString() + " is not a CONSTANCE.");
					return false;
				}
			}
			++nIndex;
			return true;
		})) {
			return false;
		}
	}
	if (m_pFieldIdentifier->m_pIdentifier->GetType() != IrisIdentifierType::Constance) {
		// ** Error **
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::IdenfierTypeIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "Identifier of " + m_pFieldIdentifier->m_pIdentifier->GetIdentifierString() + " is not a CONSTANCE.");
		return false;
	}


	return true;
}
